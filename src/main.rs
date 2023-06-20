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
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use greg_os::task::{executor::Executor, keyboard, Task};

// The entry point function to our kernel
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    greg_os::vga_buffer::print_logo();
    greg_os::init(boot_info);

    #[cfg(test)]
    test_main();

    // Asynchronous runtime executor
    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use greg_os::println;
    println!("{}", info);
    greg_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    greg_os::test_panic_handler(info)
}
