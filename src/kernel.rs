#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]


use core::panic::PanicInfo; // panic info structure

pub mod interrupts; // interrupt handling
pub mod serial;
pub mod vga;


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[unsafe(no_mangle)] // dont mangle the name of this function
pub extern "C" fn kernel_main() -> ! {
    println!("Hello World{}", "!");
    serial_print!("Hello Serial{}", "!");

    // initialize cpu interrupts
    interrupts::init_idt();

    x86_64::instructions::interrupts::int3();

    unsafe {
        *(0xDEADBEEF as *mut u8) = 42;
    }

    println!("It did not crash!");
    serial_print!("Serial Test! yippie");

    loop {}
}

