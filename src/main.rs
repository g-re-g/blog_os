// Do not auto-import the Rust standard library
#![no_std]
// Do not auto-generate a main function
#![no_main]
// Do not use the builtin rust test framework
#![feature(custom_test_frameworks)]
// Specify our custom test runner function
#![test_runner(greg_os::test_runner)]
// What name do we want to give the test harness main function
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
use alloc::boxed::Box;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use greg_os::memory::{self, BootInfoFrameAllocator};
use greg_os::task::{executor::Executor, keyboard, Task};
use greg_os::{allocator, println};
use x86_64::VirtAddr; // new

// The entry point function to our kernel
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    greg_os::vga_buffer::print_logo();
    greg_os::init();

    #[cfg(test)]
    test_main();

    // TODO: put this in an initializer somewhere else
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // TODO: turn this into a test
    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);
    drop(heap_value);
    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);
    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // Asynchronous runtime executor
    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    greg_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    greg_os::test_panic_handler(info)
}
