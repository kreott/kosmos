use x86_64::VirtAddr;
use x86_64::structures::{
    tss::TaskStateSegment,
    gdt::{
        Descriptor,
        GlobalDescriptorTable,
        SegmentSelector,
    }
};
use lazy_static::lazy_static;

// index in ist for double fault
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    // tss with interrupt stacks
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        // setup stack for double fault
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5; // 5 pages
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            // stack grows down, so use end
            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static! {
    // gdt with code and tss segments
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment()); // kernel code
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS)); // tss segment
        (gdt, Selectors { code_selector, tss_selector })
    };
}

// container for selectors
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};

    GDT.0.load(); // load gdt
    unsafe {
        CS::set_reg(GDT.1.code_selector); // switch code segment
        load_tss(GDT.1.tss_selector); // load tss
    }
}