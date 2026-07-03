# Benchmark Report: Zero-Allocation VEH Arena Server

## Overview
This report documents an experimental architecture designed to achieve Linux `fork()` style Copy-On-Write (COW) memory persistence natively on Windows without freezing a database engine. By weaponizing Windows Hardware Exceptions (Vectored Exception Handling) and bypassing global heap allocations, this architecture successfully achieved multi-million Requests Per Second (RPS) read/write throughput on Windows, outperforming native single-threaded Linux caching solutions.

## What we did with Rust
To validate the architectural theory, we built a standalone, high-performance TCP server Proof of Concept (PoC) in Rust. The implementation relied on three core principles:
1. **Zero-Allocation Flat Arena:** Instead of using the global heap (`std::alloc` or `HashMap`), the entire database was pre-allocated as a single, contiguous array of fixed-size slots (122 MB) using `VirtualAlloc`. 
2. **Lock-Free Linear Probing:** The array acted as a flat hash table. Network threads hashed incoming keys and jumped directly to the correct array index, allowing all threads to read and write concurrently without a global lock.
3. **VEH Copy-On-Write:** To simulate a background snapshot without locks, we called `VirtualProtect(PAGE_READONLY)` on the entire Arena block. When a network thread attempted a `SET` on a protected page, it triggered a hardware Page Fault (`0xC0000005`). A registered Windows Vectored Exception Handler (VEH) caught the fault, copied the 4KB page to a backup buffer, unprotected the page, and instantly resumed execution.

## Result
We benchmarked the Windows PoC against standard native Linux caching infrastructure on equivalent hardware using standard benchmarking tools (`get`/`set` operations, 100 clients, pipeline 200).

| Metric | Native Linux Baseline | Windows VEH Arena PoC | Improvement |
| :--- | :--- | :--- | :--- |
| **SET (Writes)** | 2,188,183 RPS | **5,524,862 RPS** | **~2.52x Faster** |
| **GET (Reads)** | 3,144,654 RPS | **5,102,041 RPS** | **~1.62x Faster** |

The PoC completely eliminated the traditional bottleneck of dynamic heap allocations and global locks, proving that the overhead of trapping hardware exceptions (approx. 2.5 microseconds per fault) is negligible compared to the massive performance gains of lock-free concurrency.

## Why not rust for future
While the Rust PoC achieved staggering performance using a fixed-size array, building a production-grade database with variable-sized data strings using this architecture in Rust is severely complex:
1. **The Global Allocator Problem:** In Rust, dynamic collections (like `String` or `DashMap`) allocate their internal nodes randomly on the standard application Heap using the Global Allocator. To protect only the database memory during a snapshot (and avoid accidentally protecting Tokio's networking buffers, which would cause catastrophic page faults on every incoming packet), one must write a custom, thread-safe, fragmentation-resistant, and VEH-aware `#[global_allocator]`.
2. **Custom Allocator Instability:** Rust's Custom Allocator trait (`std::alloc::Allocator`) is currently highly unstable (nightly-only) for complex collections, and many popular community crates do not support passing custom allocators at all.
3. **Fighting the Borrow Checker:** Building custom slab allocators utilizing manual offset-based pointers requires heavily bypassing the borrow checker with `unsafe` blocks, effectively neutralizing Rust's primary safety guarantees.

## Entire Idea on C++
For a production-grade commercial product, this architecture is perfectly suited for **C++**. C++ provides the necessary ecosystem (Boost.Asio) and first-class support for custom memory allocators out of the box.

### 1. The Network Layer
Use **Boost.Asio** to handle cross-platform asynchronous I/O. On Windows, Boost.Asio natively utilizes I/O Completion Ports (IOCP), providing the absolute lowest latency TCP packet handling available. Spin up an `io_context` thread pool matching the core count of the machine to parse incoming protocol commands.

### 2. The Memory Manager (Slab Allocator)
To support variable-sized strings without heap fragmentation, build a **Slab Allocator** inside a custom `VirtualAlloc` block:
- Partition the block into different size classes (16B, 32B, 64B, 128B, etc.).
- Maintain a lock-free `FreeList` (an atomic stack of memory offsets) for each size class.
- When a `SET` command arrives, pop an offset from the corresponding Slab class and write the data sequentially into the custom block.

### 3. The Index (Custom STL Allocator)
To insulate the database from the standard OS heap, instantiate a `std::unordered_map` utilizing a **Custom STL Allocator** that draws memory strictly from your `VirtualAlloc` block. The map will store offsets (integers) pointing into your Slab Allocator rather than raw heap pointers.

### 4. The VEH Trap
Register the VEH using `<windows.h>` to seamlessly trap and clone pages on demand:
```cpp
AddVectoredExceptionHandler(1, [](PEXCEPTION_POINTERS ex) -> LONG {
    if (ex->ExceptionRecord->ExceptionCode == EXCEPTION_ACCESS_VIOLATION) {
        void* fault_addr = (void*)ex->ExceptionRecord->ExceptionInformation[1];
        if (is_in_custom_arena(fault_addr)) {
            void* page = align_to_4kb(fault_addr);
            // 1. Copy page to backup buffer for BGSAVE thread
            // 2. Unprotect page: VirtualProtect(page, 4096, PAGE_READWRITE, &old)
            return EXCEPTION_CONTINUE_EXECUTION;
        }
    }
    return EXCEPTION_CONTINUE_SEARCH;
});
```

By combining Boost.Asio's IOCP networking with a Custom Slab Allocator and Windows VEH, C++ engineers can build a highly concurrent database engine that easily exceeds standard Linux caching throughput while providing zero-pause background persistence.
