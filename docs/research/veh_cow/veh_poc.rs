use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use windows_sys::Win32::Foundation::EXCEPTION_ACCESS_VIOLATION;
use windows_sys::Win32::System::Diagnostics::Debug::{AddVectoredExceptionHandler, EXCEPTION_POINTERS};
use windows_sys::Win32::System::Memory::{VirtualAlloc, VirtualProtect, MEM_COMMIT, MEM_RESERVE, PAGE_READONLY, PAGE_READWRITE};

const EXCEPTION_CONTINUE_EXECUTION: i32 = -1;
const EXCEPTION_CONTINUE_SEARCH: i32 = 0;

const PAGE_SIZE: usize = 4096;
const NUM_PAGES: usize = 100_000; // 100,000 pages = ~400MB
const TOTAL_SIZE: usize = PAGE_SIZE * NUM_PAGES;

static FAULT_COUNT: AtomicUsize = AtomicUsize::new(0);
static mut BASE_PTR: *mut u8 = std::ptr::null_mut();

unsafe extern "system" fn veh_handler(exception_info: *mut EXCEPTION_POINTERS) -> i32 {
    let record = (*exception_info).ExceptionRecord;
    if (*record).ExceptionCode == EXCEPTION_ACCESS_VIOLATION {
        let fault_addr = (*record).ExceptionInformation[1] as *mut u8;
        
        // Check if the fault is within our allocated block
        if fault_addr >= BASE_PTR && fault_addr < BASE_PTR.add(TOTAL_SIZE) {
            // Calculate the page boundary
            let offset = fault_addr as usize - BASE_PTR as usize;
            let page_offset = offset - (offset % PAGE_SIZE);
            let page_addr = BASE_PTR.add(page_offset);
            
            // In Memurai they copy the page here to a backup buffer!
            // We just unprotect it to let the write succeed.
            let mut old_protect = 0;
            VirtualProtect(page_addr as _, PAGE_SIZE, PAGE_READWRITE, &mut old_protect);
            
            FAULT_COUNT.fetch_add(1, Ordering::SeqCst);
            return EXCEPTION_CONTINUE_EXECUTION;
        }
    }
    EXCEPTION_CONTINUE_SEARCH
}

fn main() {
    unsafe {
        println!("Allocating {} MB of memory...", TOTAL_SIZE / 1024 / 1024);
        BASE_PTR = VirtualAlloc(
            std::ptr::null(),
            TOTAL_SIZE,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        ) as *mut u8;
        
        if BASE_PTR.is_null() {
            panic!("Failed to allocate memory");
        }

        println!("Registering Vectored Exception Handler...");
        let handler = AddVectoredExceptionHandler(1, Some(veh_handler));
        if handler.is_null() {
            panic!("Failed to register VEH");
        }

        println!("Protecting memory as READONLY to simulate a BGSAVE snapshot...");
        let mut old_protect = 0;
        VirtualProtect(BASE_PTR as _, TOTAL_SIZE, PAGE_READONLY, &mut old_protect);

        println!("Starting benchmark: Mutating {} pages to trigger Access Violations...", NUM_PAGES);
        let start = Instant::now();
        
        for i in 0..NUM_PAGES {
            let p = BASE_PTR.add(i * PAGE_SIZE);
            // This write WILL crash, trigger VEH, get unprotected, and resume!
            std::ptr::write_volatile(p, 42); 
        }
        
        let elapsed = start.elapsed();
        
        println!("Benchmark Complete!");
        println!("Hardware Page Faults Caught & Resumed: {}", FAULT_COUNT.load(Ordering::SeqCst));
        println!("Time taken: {:?}", elapsed);
        println!("Time per fault: {:?}", elapsed / NUM_PAGES as u32);
        
        let ops_per_sec = (NUM_PAGES as f64 / elapsed.as_secs_f64()) as usize;
        println!("Max Copy-On-Write Faults/sec: {}", ops_per_sec);
    }
}
