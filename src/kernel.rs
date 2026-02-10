#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc; // rust alloc

use core::panic::PanicInfo; // panic info structure
use crate::{bootinfo::BootInfo}; // boot info structure
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc}; // stuff


pub mod interrupts; // interrupt handling
pub mod serial; // serial output
pub mod vga; // vga output
pub mod gdt; // gdt handling
pub mod bootinfo; // boot info sent to _start()
pub mod memory; // memory management
pub mod allocator;


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
    use crate::memory::BootInfoFrameAllocator;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_regions)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    println!("We got out of initializing the heap");
    
    // allocate number on heap
    let heap_value = Box::new(41);
    println!("heap value at {:p}", &*heap_value);
    // create vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector. will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference)); 
    

    println!("It did not crash! :D");
    hlt_loop();
} // fn kernel_main