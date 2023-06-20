#![allow(clippy::new_without_default)]
use crate::println;
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::stream::Stream;
use futures_util::task::AtomicWaker;

/// A struct for waking up a sleeping task
/// https://doc.rust-lang.org/std/task/struct.Waker.html
/// https://docs.rs/futures-util/latest/futures_util/task/struct.AtomicWaker.html
static WAKER: AtomicWaker = AtomicWaker::new();

/// A static queue for holding keyboard scancodes
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
/// The max number of keyboard scancodes we can have in the queue
const SCANCODE_QUEUE_LENGTH: usize = 100;

/// A type for initializing and using the SCANCODE_QUEUE
pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(SCANCODE_QUEUE_LENGTH))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

/// Implementing the Stream trait for a stream of keyboard scancodes
/// https://docs.rs/futures-util/latest/futures_util/stream/trait.Stream.html
impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("SCANCODE_QUEUE not initialized");

        // fast path
        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(cx.waker());

        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

/// WARNING Called by the keyboard interrupt handler.
/// WARNING Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if queue.push(scancode).is_err() {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

// pub async fn print_keypresses() {
//     use crate::print;
//     use futures_util::stream::StreamExt;
//     use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

//     let mut scancodes = ScancodeStream::new();
//     let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

//     while let Some(scancode) = scancodes.next().await {
//         if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
//             if let Some(key) = keyboard.process_keyevent(key_event) {
//                 let dt = crate::rtc::read_rtc();

//                 println!("{:?}", dt);

//                 match key {
//                     DecodedKey::Unicode(character) => print!("{}", character),
//                     DecodedKey::RawKey(key) => print!("{:?}", key),
//                 }
//             }
//         }
//     }
