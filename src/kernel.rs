#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

// imports
extern crate alloc; // rust alloc
use core::panic::PanicInfo; // panic info structure
use crate::{bootinfo::BootInfo, task::keyboard}; // boot info structure
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc}; // stuff
use crate::task::{Task, simple_executor::SimpleExecutor};
use x86_64::VirtAddr; // virtual address struct
use crate::memory::BootInfoFrameAllocator; // boof info frame allocator

// modules
pub mod interrupts; // interrupt handling
pub mod serial; // serial output
pub mod vga; // vga output
pub mod gdt; // gdt handling
pub mod bootinfo; // boot info sent to _start()
pub mod memory; // memory management
pub mod allocator; // memory allocator
pub mod task; // async tasks


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
    // initialize heap
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_regions)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");
    println!("initialized heap");

    vgaclear!();
    println!("Hello World{}", "!");
    serial_print!("Hello Serial{}", "!");
    
    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    println!("It did not crash! :D");
    hlt_loop();
} // fn kernel_main