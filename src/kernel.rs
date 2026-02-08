#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo; // panic info structure
use crate::{bootinfo::BootInfo, memory::translate_addr}; // boot info structure


pub mod interrupts; // interrupt handling
pub mod serial; // serial output
pub mod vga; // vga output
pub mod gdt; // gdt handling
pub mod bootinfo; // boot info sent to _start()
pub mod memory; // memory management


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn init() {
    gdt::init(); // init gdt
    interrupts::init_idt(); // init idt
    unsafe { interrupts::PICS.lock().initialize() }; // init PICS
    x86_64::instructions::interrupts::enable(); // enable interrupts
}

#[unsafe(no_mangle)] // dont mangle the name of this function
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {

    // initialize important things like gdt and interrupts
    init();

    vgaclear!();
    println!("Hello World{}", "!");
    serial_print!("Hello Serial{}", "!");

    use x86_64::VirtAddr;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset.into_option().unwrap(),
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        println!("lets see what happens!!!");
        let phys = unsafe { translate_addr(virt, phys_mem_offset) };
        println!("{:?} -> {:?}", virt, phys);
    }

    println!("It did not crash! :D");
    hlt_loop();
} // fn kernel_main