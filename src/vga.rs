use core::fmt::{Write, Result, self}; // for Result and stuff
use volatile::Volatile; // for volatile, dont want to optimize our vga writes away
use lazy_static::lazy_static; // for lazystatic obviously
use spin::Mutex; // for mutex
use x86_64::instructions::port::Port; // for port functionality

// VGA color values
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // make sure each enum is u8
pub enum Color {
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
    White = 15
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        // VGA packs bg in the high nibble, fg in the low one
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

// this maps directly onto VGA text memory
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    row_position: usize, // tracked for future cursor stuff
    pub color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                // write to the current row and col
                let row = self.row_position;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });

                self.column_position += 1;
                self.sync_cursor();
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII or newline
                0x20..=0x7E | b'\n' => self.write_byte(byte),
                // replace anything weird with a block
                _ => self.write_byte(0xFE),
            }
        }
    }

    // backspace handler
    pub fn backspace(&mut self) {
        if self.column_position > 0 {
            self.column_position -= 1;
            let row = self.row_position;
            let col = self.column_position;

            self.buffer.chars[row][col].write(ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            });

            self.sync_cursor();
        }
    }

    fn new_line(&mut self) {
        self.column_position = 0;

        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            // scroll one row if at top
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }

            // clear the bottom row after scrolling
            self.clear_row(BUFFER_HEIGHT - 1);
        }
        self.sync_cursor();
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }

        self.column_position = 0;
        self.row_position = 0;
        self.sync_cursor();
    }

    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }

    fn sync_cursor(&self) {
        update_hardware_cursor(self.row_position, self.column_position);
    }
} // impl Writer

// function for updating the actual hardware vga cursors position as we print with vga
fn update_hardware_cursor(row: usize, col: usize) {
    let pos = row * BUFFER_WIDTH + col;
    let pos = pos as u16;

    unsafe {
        let mut cmd: Port<u8> = Port::new(0x3D4);
        let mut data: Port<u8> = Port::new(0x3D5);

        // low byte
        cmd.write(0x0F);
        data.write((pos & 0xFF) as u8);

        // high byte
        cmd.write(0x0E);
        data.write((pos >> 8) as u8);

    }
}

// lets Writer work with write! / format_args!
impl Write for Writer {
    fn write_str(&mut self, s: &str) -> Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    // global writer locked behind a spinlock
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        // VGA text buffer lives at 0xb8000
        buffer: unsafe { &mut *(0xB8000 as *mut Buffer) },
    });
}

// print without a newline
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

// print and then move to the next line
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! vgaclear {
    () => ($crate::vga::_clear());
}

#[macro_export]
macro_rules! printcolor {
    ($fg:expr, $bg:expr, $($arg:tt)*) => {{
        use crate::vga::{WRITER, Color, ColorCode};
        let mut writer = WRITER.lock();

        // save the previous color
        let prev_color = writer.color_code;

        // set new color
        writer.color_code = ColorCode::new($fg, $bg);

        // print the formatted text
        core::fmt::write(&mut *writer, format_args!($($arg)*)).unwrap();

        // restore previous color
        writer.color_code = prev_color;
    }};
}

#[doc(hidden)]
pub fn _clear() {
    WRITER.lock().clear_screen();
}


// Prints the given formatted string to the VGA text buffer
// through the global 'WRITER' instance.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}