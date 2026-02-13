use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::sync::atomic::AtomicBool;
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::{AtomicWaker};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use alloc::string::String;
use crate::task::Ordering;
use crate::print;

/// PS/2 queue & waker
static PS2_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static PS2_WAKER: AtomicWaker = AtomicWaker::new();

/// USB queue & waker
static USB_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static USB_WAKER: AtomicWaker = AtomicWaker::new();

// USB active flag
static USB_ACTIVE: AtomicBool = AtomicBool::new(true);


/// Run once during kernel init
pub fn init_keyboard_stream() {
    let _ = PS2_QUEUE.try_init_once(|| ArrayQueue::new(100));
    let _ = USB_QUEUE.try_init_once(|| ArrayQueue::new(100));
}

/// Called by the PS/2 or USB keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_ps2_scancode(scancode: u8) {
    if USB_ACTIVE.load(Ordering::Relaxed) {
        return;
    }

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

pub(crate) fn add_usb_scancode(scancode: u8) {
    if !USB_ACTIVE.load(Ordering::Relaxed) {
        USB_ACTIVE.store(true, Ordering::Relaxed);
        return;
    }

    if let Ok(queue) = USB_QUEUE.try_get() {
        if queue.push(scancode).is_err() {
            panic!("USB queue full; dropping input")
        } else {
            USB_WAKER.wake();
        }
    } else {
        panic!("USB queue uninitialized");
    }
}

// Async stream that merges PS/2 + USB
pub struct CombinedStream {
    _private: (),
}

impl CombinedStream {
    pub fn new() -> Self {
        CombinedStream { _private: () }
    }
}

impl Stream for CombinedStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<u8>> {
        // try PS/2 first
        if let Ok(queue) = PS2_QUEUE.try_get() {
            if let Some(sc) = queue.pop() {
                return Poll::Ready(Some(sc));
            }
            PS2_WAKER.register(&cx.waker());
        }

        // then try USB
        if let Ok(queue) = USB_QUEUE.try_get() {
            if let Some(sc) = queue.pop() {
                return Poll::Ready(Some(sc));
            }
            USB_WAKER.register(&cx.waker());
        }

        Poll::Pending
    }
}

// Read a line from the keyboard until Enter is pressed
pub async fn get_line() -> String {
    let mut scancodes = CombinedStream::new();
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
    } // loop
    line
} // async fn get_line
