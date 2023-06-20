// Tools for working with the standard VGA buffer. Namely the macros `print!`
// and `println!`.
//
// more info:
//    https://wiki.osdev.org/Printing_To_Screen
//    https://en.wikipedia.org/wiki/VGA_text_mode
//    https://en.wikipedia.org/wiki/Code_page_437
use lazy_static::lazy_static;
use spin::Mutex;

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

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
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

// Convenience wrapper around a 2 byte VGA color code and a VGA character
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

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

    // TODO: total hack to get scrolling upward working. write a proper text
    // buffer rendered mode
    pub fn pre_write_line(&mut self, s: &str) {
        // Clear the row and move text down
        use x86_64::instructions::interrupts;
        interrupts::without_interrupts(|| {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    unsafe {
                        // let character = self.buffer.chars[row][col].read();
                        let character = core::ptr::read_volatile(
                            &self.buffer.chars[BUFFER_HEIGHT - row - 1][col],
                        );
                        // self.buffer.chars[row - 1][col].write(character);
                        core::ptr::write_volatile(
                            &mut self.buffer.chars[BUFFER_HEIGHT - row][col],
                            character,
                        );
                    }
                }
            }
            self.clear_row(0);
            self.column_position = 0;
            let color_code = self.color_code;

            for byte in s.bytes() {
                let col = self.column_position;

                // self.buffer.chars[row][col].write(ScreenChar {
                //     ascii_character: byte,
                //     color_code,
                // });
                unsafe {
                    core::ptr::write_volatile(
                        &mut self.buffer.chars[0][col],
                        ScreenChar {
                            ascii_character: byte,
                            color_code,
                        },
                    );
                }

                self.column_position += 1;
            }
            self.column_position = 0;
        });
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.write_byte(*byte)
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

    pub fn clear_screen(&mut self) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                unsafe {
                    core::ptr::write_volatile(&mut self.buffer.chars[row][col], blank);
                }
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

pub fn center(w: &mut Writer, s: &str) {
    let left_padding = BUFFER_WIDTH / 2 - s.len() / 2;
    for _ in 0..left_padding {
        w.write_byte(b' ');
    }
    w.write_string(s)
}

const CURSOR_ADDRESS_REGISTER: u16 = 0x3D4;
const CURSOR_DATA_REGISTER: u16 = 0x3D5;

pub fn disable_cursor() {
    use x86_64::instructions::port::Port;
    let mut address: Port<u8> = Port::new(CURSOR_ADDRESS_REGISTER);
    let mut data: Port<u8> = Port::new(CURSOR_DATA_REGISTER);
    unsafe {
        address.write(0x0A);
        data.write(0x20);
    }
}

pub fn enable_cursor() {
    use x86_64::instructions::port::Port;
    let mut address: Port<u8> = Port::new(CURSOR_ADDRESS_REGISTER);
    let mut data: Port<u8> = Port::new(CURSOR_DATA_REGISTER);
    // Starting row
    let cursor_start = 1;
    // Ending row
    let cursor_end = 2;
    unsafe {
        address.write(0x0A);
        let b = data.read();
        data.write((b & 0xC0) | cursor_start);

        address.write(0x0B);
        let b = data.read();
        data.write((b & 0xE0) | cursor_end);
    }
}

pub fn move_cursor(x: usize, y: usize) {
    use x86_64::instructions::port::Port;
    let position = x + y * BUFFER_WIDTH;
    let mut address: Port<u8> = Port::new(CURSOR_ADDRESS_REGISTER);
    let mut data: Port<u8> = Port::new(CURSOR_DATA_REGISTER);

    unsafe {
        address.write(0x0F as u8);
        data.write((position & 0xFF) as u8);
        address.write(0x0E as u8);
        data.write(((position >> 8) & 0xFF) as u8);
    }
}

// This is long and ugly because it uses characters from code page 437 and I
// haven't figured out an easy way to input them or written a macro to make
// this better.
pub fn print_logo() {
    let s: &[u8] = &[b'G', b'r', b'e', b'g', 0x01, b'S'];
    let logo_width = s.len() + 4;
    let left_padding = BUFFER_WIDTH / 2 - logo_width / 2;
    let padd_left = {
        |w: &mut Writer| {
            for _ in 0..left_padding {
                w.write_byte(b' ');
            }
        }
    };
    {
        let mut w = WRITER.lock();
        w.color_code = ColorCode::new(Color::LightGreen, Color::Black);
        // Top Left
        padd_left(&mut w);
        w.write_byte(0xC9);
        // Top Bar
        for _ in 0..s.len() + 2 {
            w.write_byte(0xCD);
        }
        // Top Right
        w.write_byte(0xBB);
        w.write_byte(b'\n');
        // Top margin left
        padd_left(&mut w);
        w.write_byte(0xBA);
        // Top Margin
        for _ in 0..s.len() + 2 {
            w.write_byte(b' ');
        }
        // Top Margin right
        w.write_byte(0xBA);
        w.write_byte(b'\n');

        // Middle left
        padd_left(&mut w);
        w.write_byte(0xB6);
        // Middle margin
        w.write_byte(b' ');
        // Middle
        w.write_bytes(s);
        // middle margin
        w.write_byte(b' ');
        // Middle right
        w.write_byte(0xC7);
        w.write_byte(b'\n');
        // Bottom margin left
        padd_left(&mut w);
        w.write_byte(0xBA);
        // Bottom Margin
        for _ in 0..s.len() + 2 {
            w.write_byte(b' ');
        }
        // Bottom Margin right
        w.write_byte(0xBA);
        w.write_byte(b'\n');
        // Bottom left
        padd_left(&mut w);
        w.write_byte(0xC8);
        // Bottom bar
        for _ in 0..s.len() + 2 {
            w.write_byte(0xCD);
        }
        // Bottom right
        w.write_byte(0xBC);
        w.write_byte(b'\n');
        padd_left(&mut w);

        w.write_bytes(&[b'\n', b'\n', b'\n']);

        w.color_code = ColorCode::new(Color::Yellow, Color::Black);
        center(&mut w, "< Press any key to continue >");

        for _ in 0..9 {
            w.write_byte(b'\n');
        }

        let time = alloc::format!("{:?}", crate::rtc::read_rtc());

        w.color_code = ColorCode::new(Color::Magenta, Color::Black);
        center(&mut w, &time[0..time.len() - 2]);

        w.color_code = ColorCode::new(Color::Yellow, Color::Black);
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
