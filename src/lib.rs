#![no_std]
#![no_main]

use core::panic::PanicInfo;
mod vga;

#[unsafe(no_mangle)] // dont mangle the name of this function
pub extern "C" fn kernel_main() -> ! {
    loop {}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}