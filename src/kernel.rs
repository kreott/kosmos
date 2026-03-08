#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

// imports
extern crate alloc; // rust alloc
use x86_64::VirtAddr; // virtual address struct
use core::panic::PanicInfo; // panic info struct
use crate::{
    bootinfo::BootInfo, // bootinfo struct
    memory::BootInfoFrameAllocator, // boot info frame allocator
    task::{ // tasks and executor
        Task, 
        keyboard,
        executor::Executor,
    },
};

// modules
pub mod interrupts; // interrupt handling
pub mod serial; // serial output
pub mod vga; // vga output
pub mod gdt; // gdt handling
pub mod bootinfo; // boot info sent to _start()
pub mod memory; // memory management
pub mod allocator; // memory allocator
pub mod task; // async tasks
pub mod timer; // PIT timer
pub mod system; // system helper functions


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

fn init() {
    gdt::init();            // init gdt
    interrupts::init_idt(); // init idt
    timer::init();          // init pit timer
    unsafe { interrupts::PICS.lock().initialize() }; // init PICS
    x86_64::instructions::interrupts::enable();      // enable interrupts
}

#[unsafe(no_mangle)] // dont mangle the name of this function
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    // clear screen
    vgaclear!();

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

    // initialize keyboard driver
    keyboard::init_keyboard_stream();

    // initialize shell
    let mut executor = Executor::new();
    crate::task::shell::spawn_shell(&mut executor);
    executor.run();
    // anything past this is unreachable, but good to have as a fallback

    #[allow(unreachable_code)]
    hlt_loop();
} // fn kernel_main

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}