#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo; // panic info structure

pub mod interrupts; // interrupt handling
pub mod serial; // serial output
pub mod vga; // vga output
pub mod gdt; // gdt handling

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

pub fn init() {
    gdt::init(); // init gdt
    interrupts::init_idt(); // init idt
    unsafe { interrupts::PICS.lock().initialize() }; // init PICS
    x86_64::instructions::interrupts::enable(); // enable interrupts
}


#[unsafe(no_mangle)] // dont mangle the name of this function
pub extern "C" fn kernel_main() -> ! {

    // initialize important things like gdt and interrupts
    init();
    
    vgaclear!();
    println!("Hello World{}", "!");
    serial_print!("Hello Serial{}", "!");

    
    println!("Working vga yippie!!! :D");


    loop {
        
    }
} // fn kernel_main

