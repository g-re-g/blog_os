// Test that a stack overflow gets handled by a double fault handler
#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

use greg_os::gdt::DOUBLE_FAULT_IST_INDEX;
use greg_os::{exit_qemu, serial_print, serial_println, QemuExitCode};
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

// Setup an IDT that just has a double fault handler
lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");
    // Initialize the global descriptor table
    greg_os::gdt::init();
    // Load the interrupt descriptor table
    TEST_IDT.load();
    // trigger a stack overflow
    stack_overflow();
    // We should never get here
    panic!("Execution continued after stack overflow");
}

// This function forces a stack overflow by calling itself recursively
// and forcing no tail call elimination.
#[allow(unconditional_recursion)]
fn stack_overflow() {
    // for each recursion, the return address is pushed to the stack
    stack_overflow();
    // force a read to prevent tail call elimination
    unsafe { core::ptr::read_volatile(0 as *mut usize) };
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    greg_os::test_panic_handler(info)
}
