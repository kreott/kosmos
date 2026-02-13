use crate::task::executor::Executor;
use crate::{Task, vgaclear};
use crate::task::keyboard::get_line;
use crate::{print, println};
use alloc::string::*;
use crate::printcolor;
use crate::vga::Color;
use crate::timer;
use alloc::format;
use raw_cpuid::CpuId;
use crate::allocator;

enum Command {
    Fetch,
    Clear,
    Unknown,
}

// New star ASCII art
const STAR_ASCII: &[&str] = &[
    "  .       .",
    "       .  |  .",
    "        \\ | /    +",
    "*        \\|/",
    "    --==> * <==--   '",
    "   +     /|\\   .",
    "        / | \\",
    ".      '  |  '       *",
    "          |",
    "    .     '    .",
];

const COMMANDS: &[&str] = &[
    "FETCH",
    "CLEAR",
];


fn trim_after_ghz(s: &str) -> &str {
    if let Some(pos) = s.find("GHz") {
        &s[..pos + 3]
    } else {
        s
    }
}

fn cpuinfo() -> String {
    let cpuid = CpuId::new();

    let brand_string = cpuid
        .get_processor_brand_string()
        .map(|b| b.as_str().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string());

    let trimmed = trim_after_ghz(&brand_string.as_str());

    format!("CPU: {}", trimmed)
}

pub fn get_stats() -> [String; 4] {    
    /*** stats ***/

    // os name
    let os = "OS: Kosmos v0.0.1".to_string();

    // uptime
    let seconds = timer::uptime_seconds();
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let uptime = format!(
        "Uptime: {:02}h {:02}m {:02}s",
        hours,
        minutes,
        seconds,
        );

    // cpu info
    let cpuinfo = cpuinfo();

    // heap info
    let heapinfo = allocator::heap_stat();
    // stats array
    [
    os,
    uptime,
    cpuinfo,
    heapinfo,
    ]
} // fn get_stats

/// Prints the ASCII art with gradient colors and stats aligned
fn print_fetch(stats: &[String]) {
    let art_lines = STAR_ASCII;
    let total_lines = art_lines.len().max(stats.len());
    let art_width = 22;

    for i in 0..total_lines {
        let art = art_lines.get(i).unwrap_or(&"");
        let stat = stats.get(i).map(|s| s.as_str()).unwrap_or("");

        let mut col = 0;
        // print each character of the art with color
        for c in art.chars() {
            let color = match c {
                '*' | '>' | '<' | '\\' | '/' | '=' => Color::White,       // core stars
                ':' | '!' | '|' | '-' | '\'' | '+' => Color::Yellow, // highlights / lines
                '.' => Color::LightRed,                // twinkles
                _ => Color::Black,                     // spaces / background
            };

            printcolor!(color, Color::Black, "{}", c);
            col += 1;
        }

        // pad spaces to reach art_width + 10 padding
        while col < art_width + 3 {
            printcolor!(Color::Black, Color::Black, " ");
            col += 1;
        }

        // print the stat
        println!("{}", stat);
    } // for i in 0..total_lines
} // fn print_fetch

pub fn print_header() {
    printcolor!(Color::White, Color::Blue, "--- Kosmos ---\n\n");
    printcolor!(Color::White, Color::Black, "COMMANDS: ");
    for cmd in COMMANDS {
        printcolor!(Color::Yellow, Color::Black, "{} ", cmd);
    }
    print!("\n");
}

fn parse_command(input: &str) -> Command {
    match input {
        s if s.eq_ignore_ascii_case("fetch") => Command::Fetch,
        s if s.eq_ignore_ascii_case("clear") => Command::Clear,
        
        _ => Command::Unknown,
    }
}

// main shell loop
pub async fn shell_task() {
    print_header();
    loop {
        print!("kosmos> ");
        let input: String = get_line().await; // wait for input
        let cmd = parse_command(&input.trim());

        match cmd {
            Command::Fetch => {
                let stats = get_stats();
                print_fetch(&stats);
                print!("\n");
            }
            Command::Clear => {
                vgaclear!();
                print_header();
            }
            Command::Unknown => {
                println!("{}: unknown command", input.trim());
                print!("\n");
            }
        }
    }
} // async fn shell_task

// Helper to spawn the shell task
pub fn spawn_shell(executor: &mut Executor) {
    executor.spawn(Task::new(shell_task()));
}
