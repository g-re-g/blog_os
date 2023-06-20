use crate::keyboard::ScancodeStream;
use futures_util::stream::StreamExt;
use greg_os::{print, println, rtc};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

pub async fn main() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);
    greg_os::vga_buffer::print_logo();

    // println!("{}", TEXT);

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                let dt = rtc::read_rtc();

                println!("{:?}", dt);

                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}

pub static TEXT: &str = r###"
================================================================================

   ____                                                              _______   
  / ___|_ __ ___  __ _                                              |.-----.|  
 | |  _| '__/ _ \/ _` |                                             ||O . O||  
 | |_| | | |  __/ (_| |                                             ||_ v _||  
  \____|_|  \___|\__, |                                             '--)-(--'  
                 |___/                                             __[=== o]__ 
  ____                        _     _           _                 |:::::::::::|
 | __ )  __ _ _ __ __ _  __ _| |__ (_)_ __ ___ (_) __ _ _ __      `-=========-`
 |  _ \ / _` | '__/ _` |/ _` | '_ \| | '_ ` _ \| |/ _` | '_ \ 
 | |_) | (_| | | | (_| | (_| | | | | | | | | | | | (_| | | | |
 |____/ \__,_|_|  \__,_|\__, |_| |_|_|_| |_| |_|_|\__,_|_| |_|
                         |___/                                

 Some kind of computer programmer.

 Phone: 805-405-6638
 Email: greg@greg.work
 LinkedIn: https://linkedin.com/in/greg-baraghimian
"###;
