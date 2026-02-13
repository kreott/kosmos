use core::sync::atomic::Ordering;
use x86_64::instructions::port::Port;

use crate::interrupts::TIMER_TICKS;

pub const TIMER_HZ: u64 = 100;

// initialize the PIT
//
// Must be called once during kernel init, before enabling interrupts.
pub fn init() {
    let divisor: u16 = (1193182 / TIMER_HZ) as u16;

    unsafe {
        let mut cmd = Port::new(0x43);
        let mut data = Port::new(0x40);

        // channel 0, access low/high byte, mode 3 (square wave)
        cmd.write(0x36 as u8);
        data.write((divisor & 0xFF) as u8);
        data.write((divisor >> 8) as u8);
    }
}

// called from the timer interrupt handler
#[inline]
pub fn tick() {
    TIMER_TICKS.fetch_add(1, Ordering::Relaxed);
}

// uptime in seconds since boot
#[inline]
pub fn uptime_seconds() -> u64 {
    TIMER_TICKS.load(Ordering::Relaxed) / TIMER_HZ
}