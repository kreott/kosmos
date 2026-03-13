use crate::{
    Task, 
    allocator, 
    print, 
    println, 
    timer, 
    task::{
        executor::Executor,
        keyboard::get_line,
    }, 
    vga::{
        Color,
        set_print_color
    },
};
use alloc::{
    format, 
    string::*, 
};
use raw_cpuid::CpuId;

// STATS AND INFO

const STAR_ASCII: &[&str] = & [
    r#"                ,      "#,
    r#"              _/((     "#,
    r#"     _.---. .'   `\    "#,
    r#"   .'      `     ^ T=  "#,
    r#"  /     \       .--'   "#,
    r#" |      /       )'-.   "#,
    r#" ; ,   <__..-(   '-.)  "#,
    r#"  \ \-.__)    ``--._)  "#,
    r#"   '.'-.__.-.          "#,
    r#"     '-...-'           "#,
];

fn print_fetch(stats: &[String]) {
    let art_lines = STAR_ASCII;
    let total_lines = art_lines.len().max(stats.len());

    for i in 0..total_lines {
        let art = art_lines.get(i).unwrap_or(&"");
        let stat = stats.get(i).map(|s| s.as_str()).unwrap_or("");

        // print art with colors
        for c in art.chars() {
            print!("{}", c);
        }

        // print stat right after art
        println!("{}", stat);
    }
}

fn cpuinfo() -> String {
    let cpuid = CpuId::new();
    
    let brand = cpuid
        .get_processor_brand_string()
        .map(|b| b.as_str().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string());
    
    let trimmed = brand
        .split_once("GHz")
        .map(|(before, _)| format!("{}GHz", before))
        .unwrap_or(brand.clone());
    
    format!("CPU: {}", trimmed)
}

pub fn get_stats() -> [String; 4] {    
    // os name
    let mut os = "OS: ".to_string();
    os.push_str(crate::system::get_os_version());

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

fn print_header() {
    set_print_color(Color::White, Color::Blue);
    print!("--- Kosmos ---\n\n");
    set_print_color(Color::White, Color::Black);
    println!("type 'help' for a list of commands");
}

// INPUT & COMMANDS

enum Command {
    Fetch,
    Clear,
    HeapTest,
    Crash,
    Reboot,
    Help,
    Unknown,
}

// COMMANDS
mod commands {
    use crate::vga::Color;
    use crate::vga::set_print_color;
    use crate::{vgaclear, system, println};
    use crate::task::shell::{get_stats, print_fetch, print_header};
    use crate::printcolor;
    use crate::allocator;
    use alloc::vec::Vec;
    use alloc::boxed::Box;

    pub fn fetch() {
        print_fetch(get_stats().as_ref());
    }

    pub fn clear() {
        vgaclear!();
        print_header();
    }

    pub fn heaptest() {
        const ALLOC_SIZE: usize = 1024;
        const TEST_ITERATIONS: usize = 250;

        let mut allocations = Vec::new();

        for i in 1..=TEST_ITERATIONS {
            allocations.push(Box::new([0u8; ALLOC_SIZE]));

            println!("Iteration: {}. Total: {} KB", i, i);
        }

        let heapstat = allocator::heap_stat();
        printcolor!(Color::LightGreen, Color::Black, "Test done: {}\n", heapstat);
        println!("Freeing memory...");
        drop(allocations);
        println!("Done!");
    }

    pub fn crash() {
        let mut count = 0;
        loop {
            let _box = Box::new([0u8; 10240]);
            Box::leak(_box);
            count += 1;
            printcolor!(Color::LightRed, Color::Black, "Allocating 10 KB...\n");
            println!("iteration: {}, total {}", count, crate::allocator::heap_stat());
        }
    }

    pub fn reboot() {
        system::reboot();
    }

    pub fn help() {
        set_print_color(Color::Yellow, Color::Black);
        println!("    fetch");
        println!("    clear");
        println!("    heap test");
        println!("    crash");
        println!("    reboot");
        set_print_color(Color::White, Color::Black);
    }

    pub fn unknown_command(input: &str) {
        if input.trim() == "" {
            return;
        }
        println!("unknown command: {}", input)
    }
} // mod commands


fn parse_input(input: &String) -> Command {
    match input.trim().to_lowercase().as_str() {
        "fetch"     => Command::Fetch,
        "clear"     => Command::Clear,
        "heap test" => Command::HeapTest,
        "crash"     => Command::Crash,
        "reboot"    => Command::Reboot,
        "help"      => Command::Help,
        _           => Command::Unknown,
    }
}

async fn shell_task() {
    print_header();
    loop {
        print!("kosmos> ");
        let input = get_line().await;
        let cmd = parse_input(&input);
        
       
        match cmd {
            Command::Fetch      => commands::fetch(),
            Command::Clear      => commands::clear(),
            Command::HeapTest   => commands::heaptest(),
            Command::Crash      => commands::crash(),
            Command::Reboot     => commands::reboot(),
            Command::Help       => commands::help(),
            Command::Unknown    => commands::unknown_command(&input.as_str()),
        }
    }
}

pub fn spawn_shell(executor: &mut Executor) {
    executor.spawn(Task::new(shell_task()))
}