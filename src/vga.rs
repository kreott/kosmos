
const WIDTH: usize = 80;
const HEIGHT: usize = 25;
const VGA_BUFFER: *mut u16 = 0xB8000 as *mut u16;
static mut VGA_COLOR: u8 = 0x0F;

// struct for cursor/next character placement and logic, also moves the actual vga software cursor
struct Cursor {
    x: usize,
    y: usize,
}
pub unsafe fn update_cursor() {
    let pos = CURSOR.y * WIDTH + CURSOR.x;

    // low byte
    core::arch::asm!("out dx, al",
            in("dx") 0x3D4u16, 
            in("al") (0x0F as u8)
    );
    core::arch::asm!("out dx, al",
        in("dx") 0x3D5u16, 
        in("al") (pos & 0xFF) as u8
    );

    // high byte
    core::arch::asm!("out dx, al",
        in("dx") 0x3D4u16, 
        in("al") (0x0E as u8)
    );
    core::arch::asm!("out dx, al", 
        in("dx") 0x3D5u16, 
        in("al") ((pos >> 8) & 0xFF) as u8
    );
}

static mut CURSOR: Cursor = Cursor { x: 0, y: 0 };

#[repr(u8)] // ensure each value is stored as a u8
pub enum Color { // enum for all vga colors
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

// helper function for print, returns color and character as a 16 bit value
unsafe fn make_entry(c: char) -> u16 { 
    ((VGA_COLOR as u16) << 8) | (c as u16)
}

// print one character
pub unsafe fn put_char(c: char) {
    // handle newline
    if c == '\n' {
        CURSOR.y += 1;
        CURSOR.x = 0;
        update_cursor();
        return;
    }

    let index = (CURSOR.y as usize) * WIDTH + (CURSOR.x as usize);
    *VGA_BUFFER.add(index) = make_entry(c);

    CURSOR.x += 1;

    // wrap to next line
    if CURSOR.x >= WIDTH {
        CURSOR.x = 0;
        CURSOR.y += 1;
    }

    // todo: implement scroll() function

    update_cursor();
}

// full print function, works with string slices only for now
pub unsafe fn print(str: &str) {
    for c in str.chars() {
        put_char(c);
    }
}

pub unsafe fn print_color(foreground: Color, background: Color) {
    VGA_COLOR = ((background as u8) << 4) | (foreground as u8);
}

pub unsafe fn clear() {
    for i in 0..(HEIGHT * WIDTH) {
        *VGA_BUFFER.add(i) = make_entry(' ');
    }
    CURSOR.x = 0;
    CURSOR.y = 0;
}