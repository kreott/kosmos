pub mod vga {

    const WIDTH: usize = 80;
    const HEIGHT: usize = 25;
    const VGA_BUFFER: *mut u16 = 0xB8000 as *mut u16;
    static mut COLOR: u8 = 0x0F;
    static mut 


    // helper function for print
    unsafe fn make_entry(c: char) -> u16 { 
        ((color as u16) << 8) | (c as u16)
    }

    pub unsafe fn put_char(c: char) {

    }
}