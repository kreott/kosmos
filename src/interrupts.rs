use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
    PageFaultErrorCode,
};
use crate::{println, gdt};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use core::sync::atomic::AtomicU64;
use spin;

// counts the number of timer ticks since boot
pub static TIMER_TICKS: AtomicU64 = AtomicU64::new(0);

// pic setup
pub const PIC_1_OFFSET: u8 = 32;          // offset for master pic
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8; // offset for slave pic

// chained pics with mutex for safe access
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// interrupt indexes for easy reference
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,  // timer interrupt
    Keyboard,              // keyboard interrupt
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

// global interrupt descriptor table
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // called when int3 is hit (breakpoint)
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        
        // called on a double fault (very bad)
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        // timer interrupt
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);

        // keyboard interrupt
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);

        // page fault exception
        idt.page_fault.set_handler_fn(page_fault_handler);

        idt
    };
}

// load the idt into the cpu
pub fn init_idt() {
    IDT.load();
}

// breakpoint exception handler
extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame
) {
    println!("exception: breakpoint\n{:#?}", stack_frame);
}

// double fault handler, never returns
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64
) -> ! {
    panic!("exception: double fault\n{:#?}", stack_frame);
}

// timer interrupt handler
extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    // increment timer tick count
    crate::timer::tick();

    // notify pic that interrupt is handled
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

// keyboard interrupt handler
extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    // global keyboard state with mutex for safe access
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(ScancodeSet1::new(),
                layouts::Us104Key, HandleControl::Ignore)
            );
    }

    // read scancode from port 0x60
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    use crate::task::keyboard;

    // push scancode into both ps2 and usb queues
    keyboard::add_ps2_scancode(scancode);

    // temporarily remove usb cause it was causing issues
    //keyboard::add_usb_scancode(scancode);

    // notify pic that interrupt is handled
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
} // fn keyboard_interrupt_handler

// page fault handler
extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    println!("page fault");
    println!("error code: {}", error_code.bits() as u64);

    // halt cpu forever after page fault
    loop {
        x86_64::instructions::hlt();
    }
}