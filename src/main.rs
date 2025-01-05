use std::{env, fs};
use std::process;
use raylib::*;
use raylib::color::Color;
use raylib::prelude::RaylibDraw;

const MY_BLUE: &str = "\x1b[34m";   // Blue color
const MY_GREEN: &str = "\x1b[32m";  // Green color
const MY_YELLOW: &str = "\x1b[33m"; // Yellow color
const RESET: &str = "\x1b[0m";      // Reset color

struct Chip {
    memory:     [u8; 4096],
    w_buffer:   [[bool; 64]; 32],
    gpr:        [u8; 16],
    stack:      [u16; 16],
    pc:         u16,
    index:      u16,
    d_timer:    u8,
    s_timer:    u8,
    fonts:      [u8; 80]
}

fn init_cpu() -> Chip {
    // Defaults
    let mut chip = Chip {
        memory:     [0u8; 4096],
        w_buffer:   [[false; 64]; 32],
        gpr:        [0u8; 16],
        stack:      [0u16; 16],
        pc:         0,
        index:      0,
        d_timer:    0,
        s_timer:    0,
        fonts:      [0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
                     0x20, 0x60, 0x20, 0x20, 0x70, // 1
                     0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
                     0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
                     0x90, 0x90, 0xF0, 0x10, 0x10, // 4
                     0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
                     0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
                     0xF0, 0x10, 0x20, 0x40, 0x40, // 7
                     0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
                     0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
                     0xF0, 0x90, 0xF0, 0x90, 0x90, // A
                     0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
                     0xF0, 0x80, 0x80, 0x80, 0xF0, // C
                     0xE0, 0x90, 0x90, 0x90, 0xE0, // D
                     0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
                     0xF0, 0x80, 0xF0, 0x80, 0x80], // F
    };
    /*
        Load Fonts:
        Loads Fonts in Memory starting at location 0x50
     */
    for i in 0..chip.fonts.len(){
        chip.memory[i+80] =chip.fonts[i];
    }
    /*
        Read Load Rom:
        Parses CLI Arguments
        ARG[1] : Rom_Name
        Reads Rom_Name and stores contents into Memory
     */
    let args: Vec<String> = env::args().collect();
    let rom = args.get(1).expect("Error: No ROM file specified").to_string();
    let rom_path = format!("roms/{}", rom);
    let rom_data = fs::read(&rom_path).expect("Error: Could not read ROM file");
    for j in 0..rom_data.len() {
        chip.memory[j + 512] = rom_data[j];
    }

    // Register Initializations
    chip.pc = 512;

    chip
}

fn execute(chip: &mut Chip){
    // Fetch
    let inst = ((chip.memory[chip.pc as usize] as u16) << 8) | (chip.memory[(chip.pc + 1) as usize] as u16);
    println!("Fetched instruction: 0x{}{:04X}{} at PC: 0x{:03X}", if inst == 0x0000 {MY_BLUE} else {MY_GREEN}  ,inst,RESET, chip.pc);
    chip.pc += 2;
    // Decode
    let opcode : u16  = (inst & 0xF000) >> 12;
    let x : u16 = (inst & 0x0F00) >> 8;
    let y : u16 = (inst & 0x00F0) >> 4;
    let n : u16 = inst & 0x000F;
    let nn : u16 = inst & 0x00FF;
    let nnn : u16 = inst & 0x0FFF;

    println!("Opcode : 0x{:04X}",opcode);   //0xF000
    println!("X :      0x{:04X}",x);        //0x0F00
    println!("Y :      0x{:04X}",y);        //0x00F0
    println!("N :      0x{:04X}",n);        //0x000F
    println!("NN :     0x{:04X}",nn);       //0x00FF
    println!("NNN :    0x{:04X}",nnn);      //0x0FFF
    //Execute
    match inst {

        _ => {}
    }



}

fn mem_dump(memory: &[u8; 4096], exit_flag: i32) {
    println!("Memory Dump:");

    for j in (0..4096).step_by(16) {
        print!("0x{:04X} | ", j);
        for k in 0..16 {
            let value = memory[j + k];
            if value == 0x00 {
                print!("{}{:02X}{} ", MY_BLUE, value, RESET);
            } else if j + k >= 0x0200 {
                print!("{}{:02X}{} ", MY_GREEN, value, RESET);
            } else {
                print!("{}{:02X}{} ", MY_YELLOW, value, RESET);
            }
        }
        println!("|");
    }

    if exit_flag > 0 {
        process::exit(exit_flag);
    }
}

fn main() {
    let mut chip = init_cpu();
    //mem_dump(&chip.memory, 0);

    let (mut rl, thread) = raylib::init()
        .size(640,320)
        .title("Chip 8")
        .build();

    let mut last_update = std::time::Instant::now();
    let update_interval = std::time::Duration::from_millis(16);

    while !rl.window_should_close(){

        let now = std::time::Instant::now();
        if now.duration_since(last_update) >= update_interval {
            execute(&mut chip); // Process a CPU cycle
            last_update = now;
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);
    }

}
