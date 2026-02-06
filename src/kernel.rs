#![no_std]
#![no_main]
#![allow(unsafe_op_in_unsafe_fn)] // fixes static mut variables giving warnings

use core::panic::PanicInfo; // panic info structure

pub mod vga;
pub mod shell;

#[unsafe(no_mangle)] // dont mangle the name of this function
pub extern "C" fn kernel_main() -> ! {
    

    

    loop {}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}