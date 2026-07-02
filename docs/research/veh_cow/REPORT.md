# Technical Reference: Windows User-Mode Copy-On-Write (COW) via Vectored Exception Handling

## Executive Summary
This document outlines a theoretical architecture for achieving Linux `fork()` style Copy-On-Write (COW) memory persistence natively on Windows without freezing the database. This approach effectively re-implements a Virtual Memory manager in user space by weaponizing Windows Hardware Exceptions to intercept and copy memory pages on demand during a background snapshot.

## 1. Architectural Blueprint
To achieve lock-free, zero-pause background snapshots on Windows using a sharded or single-threaded architecture (where data is highly mutable), the database must implement a custom Global Allocator paired with a global exception handler.

### The Mechanism:
1. **The Custom Memory Pool (`VirtualAlloc`)**
   Instead of using standard heap allocators (`malloc` / `std::alloc`), the database must route all data-structure allocations through a custom memory pool. This pool is allocated as a massive, contiguous block directly against the Windows Page File using:
   ```c
   HANDLE mapping = CreateFileMappingA(INVALID_HANDLE_VALUE, NULL, PAGE_READWRITE, 0, POOL_SIZE, NULL);
   void* view = MapViewOfFile(mapping, FILE_MAP_ALL_ACCESS, 0, 0, 0);
   ```

2. **The Exception Trap (`AddVectoredExceptionHandler`)**
   The application registers a global Vectored Exception Handler (VEH) at startup. The VEH is designed to intercept and resolve hardware faults before they crash the process.

3. **Triggering the Snapshot (`VirtualProtect`)**
   When a background save initiates, the database does *not* clone any data. Instead, it issues a single OS-level call to change the permissions of the entire custom memory pool:
   ```c
   VirtualProtect(pool_base, POOL_SIZE, PAGE_READONLY, &old_protect);
   ```
   A background thread is then spawned to sequentially read this memory pool and serialize it to disk.

4. **The Hardware Fault (`EXCEPTION_ACCESS_VIOLATION`)**
   If a client attempts to mutate data while the background save is running, the main execution thread will try to write to a page marked `PAGE_READONLY`. The CPU instantly halts the thread and triggers a hardware Page Fault (`0xC0000005`).

5. **The Rescue and Bifurcation (The VEH Logic)**
   The registered VEH catches the exception and executes the following logic:
   - **Verify the Fault:** Confirm the exception code is `0xC0000005` and the faulting address lies within the protected memory pool.
   - **Isolate the Page:** Calculate the exact 4KB memory page boundary where the fault occurred.
   - **Pause and Copy:** Acquire an `SRWLock` to freeze other potential mutating threads, and `memcpy` the faulting 4KB page to a secondary shadow buffer. This preserves the original, pre-mutation state of the page for the background thread to serialize safely.
   - **Unprotect the Page:** Call `VirtualProtect(page_address, 4096, PAGE_READWRITE, &old)` to restore write access to that specific page.
   - **Resume Execution:** Return `EXCEPTION_CONTINUE_EXECUTION`. The OS resumes the main thread, which successfully executes the write instruction without ever realizing it was paused.

## 2. Experimental Validation
A standalone Rust Proof of Concept was developed to benchmark the viability and overhead of this theoretical mechanism.

**Benchmark Metrics:**
- **Scenario:** 100,000 discrete 4KB pages allocated, protected, and sequentially mutated to trigger a relentless storm of hardware faults.
- **Overhead:** ~2.5 microseconds per fault resolution.
- **Throughput:** ~400,000 Copy-On-Write page faults processed per second on a single core.

**Conclusion:** The overhead of trapping hardware exceptions and dropping into user-mode is phenomenally low. For standard caching workloads (where only a few thousand keys mutate per second during a snapshot), the latency hit is sub-millisecond and entirely invisible to end users. 

## 3. Implementation Challenges in Rust
While this architecture allows for extreme write concurrency (by enabling the use of sharded structures like `DashMap` without Write Lock Contention), implementing it in an open-source Rust project introduces severe engineering complexities:

### The Global Allocator Segmentation Problem
In C, routing database allocations through a custom function (like `zmalloc`) while leaving network buffers and event loops on standard `malloc` is trivial. 

In Rust, dynamic collections (like `DashMap` or `String`) allocate their internal nodes randomly on the standard application Heap using the Global Allocator. To protect only the database memory (and avoid accidentally protecting Tokio's networking buffers, which would cause catastrophic page faults on every incoming packet), one must write a custom, thread-safe, fragmentation-resistant, and VEH-aware `#[global_allocator]`. 

Furthermore, Rust's Custom Allocator trait (`std::alloc::Allocator`) is currently highly unstable (nightly-only) for complex collections.

### Final Verdict
Implementing User-Mode COW via VEH is a brilliant and highly performant architecture for Windows-native systems programming. However, it requires a complete commitment to writing custom memory allocators. For projects prioritizing safety, simplicity, and zero OS-coupling, utilizing an immutable, lock-free structure (such as an `im::HashMap` HAMT) provides O(1) snapshots without the need for manual memory segmentation or exception hijacking.
