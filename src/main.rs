#![no_std]
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// static HELLO: &[u8] = b"WHAT UP DOG?";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // let vga_buffer = 0xb8000 as *mut u8;

    // for (i, &byte) in HELLO.iter().enumerate() {
    //     unsafe {
    //         *vga_buffer.offset(i as isize * 2) = byte;
    //         *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
    //     }
    // }

    // let mut writer = Writer::new();
    // writer.write_byte(b'H');
    // writer.write_string("ello ");
    // writer.write_string("Wörld!");
    println!("What is up now doggerz?");

    // use core::fmt::Write;
    // write!(writer, "The numbers are {} and {}", 42, 1.0 / 3.0).unwrap();

    loop {}
}
