#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod task;
pub mod vga_buffer;
extern crate alloc;
use core::panic::PanicInfo;

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

pub fn init() {
    // Disable interrupts before we do anything
    x86_64::instructions::interrupts::disable();
    // Setup the interrupt descriptor table to handle interrupts
    interrupts::init_idt();
    // Setup the global descriptor table
    gdt::init();
    //
    unsafe { interrupts::PICS.lock().initialize() };
    // Enable interrupts
    x86_64::instructions::interrupts::enable();
}

// Trait for wrapping test functions to get some nice output
// and unify them under `dyn Testable`
pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        // print the test's nam
        serial_print!("{}...\t", core::any::type_name::<T>());
        // run the test
        self();
        // if we get here the test succeeded
        serial_println!("[ok]");
    }
}

// Run all the tests and exit with success if they all pass
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

// print the test failed and the error and exit qemu
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    // Just in case qemu doesn't exit, enter infinite halt loop
    hlt_loop();
}

/// Entry point for `cargo test`
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

// qemu applies the following math to the exit code:
// (value << 1) | 1
// so `0x10` becomes `0x21` or 31 in decimal
// so `0x11` becomes `0x23` or 35 in decimal
// what is considered success his configured in cargo.toml.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

// Utility function to quit qemu with an exit code.
// Configured on port 0xf4 in cargo.toml.
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
