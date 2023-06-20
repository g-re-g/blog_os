use crate::keyboard::ScancodeStream;
use futures_util::stream::StreamExt;
use greg_os::{print, println, vga_buffer};
use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1};

enum States {
    Home,
    Resume,
}

pub async fn main() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);
    vga_buffer::disable_cursor();
    vga_buffer::print_logo();
    let mut state = States::Home;
    let mut cursor_x = 0;
    let mut cursor_y = vga_buffer::BUFFER_HEIGHT - 1;
    let mut screen_top = 0;
    let num_lines = TEXT.lines().count();

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match state {
                    States::Home => {
                        vga_buffer::WRITER.lock().clear_screen();
                        vga_buffer::enable_cursor();
                        vga_buffer::move_cursor(cursor_x, cursor_y);
                        state = States::Resume;
                        render_resume(screen_top);
                    }
                    States::Resume => match key {
                        DecodedKey::Unicode(character) => {
                            if character == (0x1b as char) {
                                // Escape
                                vga_buffer::WRITER.lock().clear_screen();
                                state = States::Home;
                                vga_buffer::disable_cursor();
                                vga_buffer::print_logo();
                            } else {
                                print!("{}", character)
                            }
                        }
                        DecodedKey::RawKey(KeyCode::ArrowDown) => {
                            cursor_y = (cursor_y + 1).min(vga_buffer::BUFFER_HEIGHT - 1);
                            let mut new_screen_top = screen_top;
                            if cursor_y == vga_buffer::BUFFER_HEIGHT - 1 {
                                new_screen_top = (screen_top + 1).min(num_lines)
                            }
                            if new_screen_top != screen_top {
                                screen_top = new_screen_top;
                                if screen_top < num_lines - vga_buffer::BUFFER_HEIGHT {
                                    add_line(screen_top + vga_buffer::BUFFER_HEIGHT - 1)
                                }
                            }
                            vga_buffer::move_cursor(cursor_x, cursor_y);
                        }
                        DecodedKey::RawKey(KeyCode::ArrowUp) => {
                            cursor_y = (cursor_y.saturating_sub(1)).max(0);
                            let mut new_screen_top = screen_top;
                            if cursor_y == 0 {
                                new_screen_top = screen_top.saturating_sub(1);
                            }
                            if new_screen_top != screen_top {
                                screen_top = new_screen_top;
                                sub_line(screen_top);
                            }

                            vga_buffer::move_cursor(cursor_x, cursor_y);
                        }
                        DecodedKey::RawKey(KeyCode::ArrowRight) => {
                            cursor_x = (cursor_x + 1).min(vga_buffer::BUFFER_WIDTH - 1);
                            vga_buffer::move_cursor(cursor_x, cursor_y);
                        }
                        DecodedKey::RawKey(KeyCode::ArrowLeft) => {
                            cursor_x = (cursor_x.saturating_sub(1)).max(0);
                            vga_buffer::move_cursor(cursor_x, cursor_y);
                        }
                        DecodedKey::RawKey(key) => print!("{:?}", key),
                    },
                }
            }
        }
    }
}

fn render_resume(screen_top: usize) {
    for (i, line) in TEXT.lines().enumerate() {
        if i >= screen_top + vga_buffer::BUFFER_HEIGHT {
            break;
        }
        if i >= screen_top {
            println!("{line}")
        }
    }
}

fn add_line(line: usize) {
    println!("{}", TEXT.lines().nth(line).unwrap_or(""))
}

fn sub_line(line: usize) {
    vga_buffer::WRITER
        .lock()
        .pre_write_line(TEXT.lines().nth(line + 1).unwrap_or(""));
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


================================================================================
= ABOUT                                                                        =
================================================================================

A passionate computer programmer with a knack for reading the documentation and
figuring things out. Over the last 15+ years of my career I have moved further
and further down the abstraction layers starting with ActionScript and
finding myself recently in more and more Rust and Assembly. I'm motivated by
hard problems and great teams and the never-stop-learning attitude of computer
programming is what continually inspires me to push myself forward in my 
career and passion for computers. I think great tools and documentation are
extremely important and having been a teacher and mentor at a tech school I
enjoy and understand the effectiveness of being able to explain things simply.


================================================================================
= EXPERIENCE                                                                   =
================================================================================

~2016-2022~
Senior Software Engineer -at- Enbala Power Networks
Elixir, Elm, Rust, Postgres, GraphQl, and many other hats

I was brought on to Enbala to help design and build the next generation of
automated electrical grid event solving. We took in grid and device telemetry,
reports of a surplus or deficit of energy and generated control signals to solve
these events by controlling thousands of devices in the field. As one of the
first engineers on the project I helped design and implement virtually all of 
its systems. This includes everything from our best in class Elm frontend, our
high throughput GraphQl API, our realtime grid-representing Elixir data model
and API server, our many 3rd party control systems integrations, our event
solving infrastructure and implementation, and our fork and  usage of the
IEC 60870 protocol library for communication with standard electrical grid
devices. I helped bring the software from no lines of code written to getting
sold to Fortune 1000 company Generac.


~2015~
Teacher and Mentor -at- Operation Spark code school

I spent a year in New Orleans, writing software, writing curriculum, teaching,
and mentoring at Operation Spark. It was an incredibly impactful and rewarding
experience where I got to learn about and express many of the things that make
me passionate about computing and this career, and also help folks understand
that with a little motivation and the right foundation they have the capacity
to do hard things and have a lot of fun learning.


~2013-2014~
Travel and Photo Booths

I spent much of 2013 and 2014 traveling around the US and Germany. I supported
myself mostly through building custom photo booths and associated websites for
corporate events and weddings. This is before the photo booth app boom so it was
easier to make money back then if you had the skills to pull it off. :)
Playing around with all that hardware, getting prints, filters, text messages,
an instantly available website all working in concert was extremely satisfying
and felt like the culmination of my abilities at the time. Making friends and
getting to travel through my own creations was extremely rewarding.


~2012~
Co-Founder and Co-Lead Developer, Spicy Tuna Labs LLC
Ruby, Javascript, Postgres and the like

After many stints in small startups an old co-worker and I started a tech
consultancy in New York City. Our business proposition was to take projects
rapidly from idea to MVP to shippable product. It was rapidly successful and
rewarding to get to work on different projects but I was young and got burnt out
after only a year. My co-founder went on to co-found Mirror.me.


~2011~
Web and Mobile Developer -at- K2 Media Labs
Ruby, Python, Javascript, Postgres and the like

K2 is where I cut my teeth on rapid development for startups. K2 was a tech 
incubator back when those were super popular and I was part of a small team that
brought to life many of the projects that K2 invested in and incubated. I was a
gun for hire for the various startups and would help them write their web and 
mobile user interfaces as well as the Ruby and Python backends and APIs. After
a year here I decided to start Spicy Tuna Labs with an old coworker and find
clients directly.


~2010~
Web Developer and India Team Lead -at- Lime Wire LLC
Actionscript, Javascript, Python

My time at Lime Wire was interesting. It was during the last year of their 
existing and so I was moved around a lot and did a lot of interesting jobs. I
started by developing a statistics and health monitoring platform in Flash and 
Flex. After a few months I was moved onto developing the UI for an in-house
iTunes competitor before Spotify existed. We had a medium sized, India based
team and I signed up to be the communications liaison for that team on the media
player project. This job and experience was incredibly rewarding and formative
for my career. I did a ton of learning though working with an incredibly
talented team and really began to solidify my love of this career.


================================================================================
= OPEN SOURCE                                                                  =
================================================================================

I find contributing to open source software particularly rewarding and love to
do it when I feel like I can contribute something useful. I've had a few github
usernames over the last couple years for work reasons but they are all me.

Some of my favorite contributions are:

~Gleam Language~
A completely handwritten lexer and parser:
https://github.com/gleam-lang/gleam/pull/891

Generating Erlang typespecs:
https://github.com/gleam-lang/gleam/pull/912

~Helix Editor~
Fix and improve object increment and decrement
https://github.com/helix-editor/helix/pull/4123

Updated the tutor logo :)
https://github.com/helix-editor/helix/pull/5681

~Leptos Web Framework~
Cleanups and bugfixes
https://github.com/leptos-rs/leptos/pulls?q=is%3Apr+author%3Ag-re-g


================================================================================
= MISC                                                                         =
================================================================================

Recently I have been much more interested in performance, protocols and how
hardware is orchestrated into a useful machine. This was inspired by my work on
SCADA protocols at Enbala and attempting to both write an NES emulator and a
game for it. Recently I've been getting into operating system theory and and
writing a tiny OS for x86.

I also ride motorcycles, play guitar, take photos, and go to french club.

This resume can also be viewed online at:
  https:greg.work/resume

This resume can also be run as a tiny little operating system:
  https://github.com/g-re-g/greg_os

================================================================================
"###;
