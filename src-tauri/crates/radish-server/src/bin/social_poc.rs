use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use windows_sys::Win32::Foundation::{EXCEPTION_ACCESS_VIOLATION};
use windows_sys::Win32::System::Diagnostics::Debug::{AddVectoredExceptionHandler, EXCEPTION_POINTERS};
use windows_sys::Win32::System::Memory::{VirtualAlloc, VirtualProtect, MEM_COMMIT, MEM_RESERVE, PAGE_READONLY, PAGE_READWRITE};

const EXCEPTION_CONTINUE_EXECUTION: i32 = -1;
const EXCEPTION_CONTINUE_SEARCH: i32 = 0;
const PAGE_SIZE: usize = 4096;

#[derive(Clone, Copy)]
#[repr(C)]
struct Slot {
    in_use: bool,
    key_len: u8,
    val_len: u8,
    key: [u8; 16],
    val: [u8; 109],
} // 128 bytes exact

const MAX_SLOTS: usize = 1_000_000;
const ARENA_SIZE: usize = MAX_SLOTS * std::mem::size_of::<Slot>();

static mut ARENA: *mut Slot = std::ptr::null_mut();
static BGSAVE_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
static FAULT_COUNT: AtomicUsize = AtomicUsize::new(0);

// Basic FNV-1a Hash
fn hash_key(key: &[u8]) -> usize {
    let mut hash: usize = 0xcbf29ce484222325;
    for &b in key {
        hash ^= b as usize;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

unsafe fn set_key(key: &[u8], val: &[u8]) {
    if key.len() > 16 || val.len() > 109 { return; }
    let mut idx = hash_key(key) % MAX_SLOTS;
    for _ in 0..100 { // Linear probe max 100
        let slot = ARENA.add(idx);
        // Volatile write not strictly needed, but let's do normal access. If read-only, it will page fault here!
        if !(*slot).in_use || (&(*slot).key[..key.len()] == key && (*slot).key_len as usize == key.len()) {
            (*slot).in_use = true;
            (*slot).key_len = key.len() as u8;
            (*slot).val_len = val.len() as u8;
            std::ptr::copy_nonoverlapping(key.as_ptr(), (*slot).key.as_mut_ptr(), key.len());
            std::ptr::copy_nonoverlapping(val.as_ptr(), (*slot).val.as_mut_ptr(), val.len());
            return;
        }
        idx = (idx + 1) % MAX_SLOTS;
    }
}

unsafe fn get_key(key: &[u8]) -> Option<Vec<u8>> {
    if key.len() > 16 { return None; }
    let mut idx = hash_key(key) % MAX_SLOTS;
    for _ in 0..100 {
        let slot = ARENA.add(idx);
        if !(*slot).in_use {
            return None;
        }
        if (*slot).key_len as usize == key.len() && &(*slot).key[..key.len()] == key {
            let mut v = vec![0u8; (*slot).val_len as usize];
            std::ptr::copy_nonoverlapping((*slot).val.as_ptr(), v.as_mut_ptr(), (*slot).val_len as usize);
            return Some(v);
        }
        idx = (idx + 1) % MAX_SLOTS;
    }
    None
}

unsafe extern "system" fn veh_handler(exception_info: *mut EXCEPTION_POINTERS) -> i32 {
    let record = (*exception_info).ExceptionRecord;
    if (*record).ExceptionCode == EXCEPTION_ACCESS_VIOLATION {
        let fault_addr = (*record).ExceptionInformation[1] as usize;
        let arena_start = ARENA as usize;
        let arena_end = arena_start + ARENA_SIZE;
        
        if fault_addr >= arena_start && fault_addr < arena_end {
            let page_addr = fault_addr - (fault_addr % PAGE_SIZE);
            let mut old_protect = 0;
            // Here in a real DB, we would memcpy this 4KB page to a backup buffer.
            // For the PoC, we just unprotect it to allow the write to proceed immediately.
            VirtualProtect(page_addr as _, PAGE_SIZE, PAGE_READWRITE, &mut old_protect);
            FAULT_COUNT.fetch_add(1, Ordering::SeqCst);
            return EXCEPTION_CONTINUE_EXECUTION;
        }
    }
    EXCEPTION_CONTINUE_SEARCH
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    unsafe {
        ARENA = VirtualAlloc(std::ptr::null(), ARENA_SIZE, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE) as *mut Slot;
        if ARENA.is_null() { panic!("Failed to allocate Arena"); }
        AddVectoredExceptionHandler(1, Some(veh_handler));
    }

    let listener = TcpListener::bind("127.0.0.1:6380").await.unwrap();
    println!("Arena-VEH PoC Server listening on :6380");
    println!("Arena Size: {} MB", ARENA_SIZE / 1024 / 1024);

    tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            println!("--- Triggering BGSAVE (Protecting Arena) ---");
            unsafe {
                let mut old_protect = 0;
                VirtualProtect(ARENA as _, ARENA_SIZE, PAGE_READONLY, &mut old_protect);
            }
            BGSAVE_IN_PROGRESS.store(true, Ordering::SeqCst);
            
            // Simulate saving to disk
            tokio::time::sleep(Duration::from_secs(2)).await;
            
            unsafe {
                let mut old_protect = 0;
                VirtualProtect(ARENA as _, ARENA_SIZE, PAGE_READWRITE, &mut old_protect);
            }
            BGSAVE_IN_PROGRESS.store(false, Ordering::SeqCst);
            println!("--- BGSAVE Complete. Faults Caught: {} ---", FAULT_COUNT.swap(0, Ordering::SeqCst));
        }
    });

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf = vec![0u8; 65536];
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(_) => return,
                };
                
                let s = String::from_utf8_lossy(&buf[..n]);
                let mut responses = Vec::new();
                
                let commands: Vec<&str> = s.split('*').collect();
                for cmd in commands {
                    if cmd.is_empty() { continue; }
                    if cmd.starts_with("3\r\n") { // SET
                        let parts: Vec<&str> = cmd.split("\r\n").collect();
                        if parts.len() >= 7 {
                            unsafe { set_key(parts[4].as_bytes(), parts[6].as_bytes()); }
                        }
                        responses.extend_from_slice(b"+OK\r\n");
                    } else if cmd.starts_with("2\r\n") { // GET
                        let parts: Vec<&str> = cmd.split("\r\n").collect();
                        if parts.len() >= 5 {
                            unsafe {
                                if let Some(val) = get_key(parts[4].as_bytes()) {
                                    let res = format!("${}\r\n{}\r\n", val.len(), String::from_utf8_lossy(&val));
                                    responses.extend_from_slice(res.as_bytes());
                                } else {
                                    responses.extend_from_slice(b"$-1\r\n");
                                }
                            }
                        } else {
                            responses.extend_from_slice(b"$-1\r\n");
                        }
                    }
                }
                
                if !responses.is_empty() {
                    let _ = socket.write_all(&responses).await;
                }
            }
        });
    }
}
