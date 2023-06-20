// Handle Interrupts and CPU Exceptions.
// Code in here must do the right thing but also do very little.
// Code in here can be called at any time including in the middle locking up
// resources and so must take care not to cause dead locks on resources or
// subtle bugs of that nature.
// See:
//   https://wiki.osdev.org/Exceptions
//   https://wiki.osdev.org/IRQ#Standard_ISA_IRQs
use crate::hlt_loop;
use crate::{gdt, println};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

// PIC interrupt vectors are default mapped to 0-7 and 9-15. This conflicts with
// CPU exceptions and so we must move the base of these vectors to some non used
// offset. Since 0x00 - 0x1F are reserved for exceptions we start at 0x20
// See: https://wiki.osdev.org/PIC#Protected_Mode
pub const PIC_1_OFFSET: u8 = 0x20;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt
    };
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: {:?} DOUBLE FAULT\n{:#?}",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // print!(".");
    // TODO: increment atomic usize to count uptime ticks
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    // use spin::Mutex;
    // use x86_64::instructions::port::Port;

    // lazy_static! {
    //     static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
    //         Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
    //     );
    // }

    // let mut keyboard = KEYBOARD.lock();
    // let mut port = Port::new(0x60);

    // let scancode: u8 = unsafe { port.read() };
    // if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
    //     if let Some(key) = keyboard.process_keyevent(key_event) {
    //         match key {
    //             DecodedKey::Unicode(character) => print!("{}", character),
    //             DecodedKey::RawKey(key) => print!("{:?}", key),
    //         }
    //     }
    // }
    // unsafe {
    //     PICS.lock()
    //         .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    // }
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode); // new

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}
