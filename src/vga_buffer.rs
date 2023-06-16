// Tools for working with the standard VGA buffer. Namely the macros `print!`
// and `println!`.
//
// more info:
//    https://wiki.osdev.org/Printing_To_Screen
//    https://en.wikipedia.org/wiki/VGA_text_mode
//    https://en.wikipedia.org/wiki/Code_page_437
use lazy_static::lazy_static;
use spin::Mutex;

// Static vga writer behind a mutex.
// must be `lazy_static` because we need not const functions to initialize it.
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// Enum representing the standard foreground and background VGA colors
// Convenient for greating VGA color code `u8`s
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

// Convenince wrapper for the 1 byte VGA color code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        // A vga color is 1 byte 0bXXXX XXXX
        //               background^    ^foreground
        ColorCode((background as u8) << 4 | (foreground as u8) | 0b10000000)
    }
}

// Convenience wrapper around a 2 byte VGA color code and a VGA character
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            // print everything for now
            self.write_byte(byte)
            // match byte {
            //     // printable ASCII byte or newline
            //     0x20..=0x7e | b'\n' => self.write_byte(byte),
            //     // not part of printable ASCII range
            //     _ => self.write_byte(0xfe),
            // }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                // self.buffer.chars[row][col].write(ScreenChar {
                //     ascii_character: byte,
                //     color_code,
                // });
                unsafe {
                    core::ptr::write_volatile(
                        &mut self.buffer.chars[row][col],
                        ScreenChar {
                            ascii_character: byte,
                            color_code,
                        },
                    );
                }

                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                unsafe {
                    // let character = self.buffer.chars[row][col].read();
                    let character = core::ptr::read_volatile(&self.buffer.chars[row][col]);
                    // self.buffer.chars[row - 1][col].write(character);
                    core::ptr::write_volatile(&mut self.buffer.chars[row - 1][col], character);
                }
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            // self.buffer.chars[row][col].write(blank);
            unsafe {
                core::ptr::write_volatile(&mut self.buffer.chars[row][col], blank);
            }
        }
    }
}

use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

// TODO: cleanup
pub fn print_logo() {
    let s = "GregOS";
    {
        let mut w = WRITER.lock();
        w.write_byte(0xC9);
        for _ in 0..s.len() {
            w.write_byte(0xCD);
        }
        w.write_byte(0xBB);
        w.write_byte(b'\n');
        w.write_byte(0xBA);
    }
    print!("{s}");
    {
        let mut w = WRITER.lock();
        w.write_byte(0xBA);
        w.write_byte(b'\n');
        w.write_byte(0xC8);
        for _ in 0..s.len() {
            w.write_byte(0xCD);
        }
        w.write_byte(0xBC);
        w.write_byte(b'\n');
    }
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let s = "Some test string that fits on a single line";
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");

        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i];
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
