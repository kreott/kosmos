use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use alloc::string::String;
use crate::print;

/// PS/2 queue & waker
static PS2_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static PS2_WAKER: AtomicWaker = AtomicWaker::new();

/// Run once during kernel init
pub fn init_keyboard_stream() {
    let _ = PS2_QUEUE.try_init_once(|| ArrayQueue::new(100));
}

/// Called by the PS/2 keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_ps2_scancode(scancode: u8) {
    if let Ok(queue) = PS2_QUEUE.try_get() {
        if queue.push(scancode).is_err() {
            panic!("PS/2 queue full; dropping input");
        } else {
            PS2_WAKER.wake();
        }
    } else {
        panic!("PS/2 queue uninitialized");
    }
}

pub struct PS2Stream {
    _private: (),
}

impl PS2Stream {
    pub fn new() -> Self {
        PS2Stream { _private: () }
    }
}

impl Stream for PS2Stream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<u8>> {
        if let Ok(queue) = PS2_QUEUE.try_get() {
            if let Some(sc) = queue.pop() {
                return Poll::Ready(Some(sc));
            }
            PS2_WAKER.register(&cx.waker());
        }

        Poll::Pending
    }
}

// Read a line from the keyboard until Enter is pressed
pub async fn get_line() -> String {
    let mut scancodes = PS2Stream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    let mut line = String::new();

    loop {
        let scancode = scancodes.next().await.expect("Scancode stream ended unexpectedly");

        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(c) => match c {
                        '\n' | '\r' => {
                            print!("\n");
                            break;
                        }
                        '\x08' => {
                            if !line.is_empty() {
                                line.pop();
                                crate::vga::WRITER.lock().backspace();
                            }
                        }
                        _ => {
                            line.push(c);
                            print!("{}", c);
                        }
                    },
                    DecodedKey::RawKey(_) => {}
                }
            }
        }
    }
    
    line
}