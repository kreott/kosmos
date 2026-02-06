use crate::keyboard;
use crate::vga;

unsafe fn print_header() {
    vga::clear();
    vga::print_color(vga::Color::White, vga::Color::Red);
    vga::print("--- Welcome to KosmOS ---\n\n");
    vga::print_color(vga::Color::White, vga::Color::Black);
    vga::print("FETCH\n");
}


pub unsafe fn init() {
    print_header();
}