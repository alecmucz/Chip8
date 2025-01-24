use std::{env, fs};
use std::process;
use rand::{thread_rng, Rng};
use raylib::consts::KeyboardKey::*;
use raylib::prelude::*;

const MY_BLUE: &str = "\x1b[34m";   // Blue color
const MY_GREEN: &str = "\x1b[32m";  // Green color
const MY_YELLOW: &str = "\x1b[33m"; // Yellow color
const RESET: &str = "\x1b[0m";      // Reset color

struct Chip {
    memory:     [u8; 4096],
    w_buffer:   [[bool; 64]; 32],
    gpr:        [u8; 16],
    stack:      [u16; 16],
    s_ptr:      u8,
    pc:         u16,
    index:      u16,
    d_timer:    u8,
    s_timer:    u8,
    fonts:      [u8; 80],
    keys:       [(char,u8, KeyboardKey); 16]
}

fn init_cpu() -> Chip {
    // Defaults
    let mut chip = Chip {
        memory:     [0u8; 4096],
        w_buffer:   [[false; 64]; 32],
        gpr:        [0u8; 16],
        stack:      [0u16; 16],
        s_ptr:      0,
        pc:         0,
        index:      0,
        d_timer:    0,
        s_timer:    0,
        fonts:      [0xF0, 0x90, 0x90, 0x90, 0xF0,             // 0
                     0x20, 0x60, 0x20, 0x20, 0x70,             // 1
                     0xF0, 0x10, 0xF0, 0x80, 0xF0,             // 2
                     0xF0, 0x10, 0xF0, 0x10, 0xF0,             // 3
                     0x90, 0x90, 0xF0, 0x10, 0x10,             // 4
                     0xF0, 0x80, 0xF0, 0x10, 0xF0,             // 5
                     0xF0, 0x80, 0xF0, 0x90, 0xF0,             // 6
                     0xF0, 0x10, 0x20, 0x40, 0x40,             // 7
                     0xF0, 0x90, 0xF0, 0x90, 0xF0,             // 8
                     0xF0, 0x90, 0xF0, 0x10, 0xF0,             // 9
                     0xF0, 0x90, 0xF0, 0x90, 0x90,             // A
                     0xE0, 0x90, 0xE0, 0x90, 0xE0,             // B
                     0xF0, 0x80, 0x80, 0x80, 0xF0,             // C
                     0xE0, 0x90, 0x90, 0x90, 0xE0,             // D
                     0xF0, 0x80, 0xF0, 0x80, 0xF0,             // E
                     0xF0, 0x80, 0xF0, 0x80, 0x80],            // F
        keys: [
            ('1', 0x1, KEY_ONE), ('2', 0x2, KEY_TWO), ('3', 0x3, KEY_THREE), ('4', 0xC, KEY_FOUR),
            ('Q', 0x4, KEY_Q),   ('W', 0x5, KEY_W),   ('E', 0x6, KEY_E),     ('R', 0xD, KEY_R),
            ('A', 0x7, KEY_A),   ('S', 0x8, KEY_S),   ('D', 0x9, KEY_D),     ('F', 0xE, KEY_F),
            ('Z', 0xA, KEY_Z),   ('X', 0x0, KEY_X),   ('C', 0xB, KEY_C),     ('V', 0xF, KEY_V)
        ]

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

fn execute(chip: &mut Chip, rl_handle : &mut RaylibHandle){
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

    //Execute
    match opcode {
        0x0 => {
            match nn {
                0xE0 => {     // Clear Screen
                    for i in 0..chip.w_buffer.len() {
                        for j in 0..chip.w_buffer.len(){
                            chip.w_buffer[i][j] = false;
                        }
                    }
                    println!("CLS");
                }
                0xEE => {     // Return from Subroutine
                    chip.pc = chip.stack[chip.s_ptr as usize];
                    chip.s_ptr -= 1;
                    println!("RET");
                }

                _ => {} }
        }
        0x1 => {    // Jump
            chip.pc = nnn;
            println!("JP {:04X}", nnn);
        }
        0x2 => {    //Subroutine NNN
            chip.s_ptr += 1;
            chip.stack[chip.s_ptr as usize] = chip.pc;
            chip.pc = nnn;
            println!("CALL {:04X}", nnn);
        }
        0x3 => {    //Skip Conditional
            let check : bool = chip.gpr[x as usize] == nn as u8;
            if check { chip.pc += 2; }
            println!("SE V{:04X} {}", x, check);
        }
        0x4 => {    //Skip Conditional
            let check : bool = chip.gpr[x as usize] != nn as u8;
            if check { chip.pc += 2; }
            println!("SNE V{:04X} {}", x, check);
        }
        0x5 => {    //Skip Conditional
            let check : bool = chip.gpr[x as usize] == chip.gpr[y as usize];
            if check { chip.pc += 2; }
            println!("SE V{:04X} {}", x, check);
        }
        0x6 => {
            chip.gpr[x as usize] = nn as u8;
            println!("LD V{:04X}", x);
        }
        0x7 => {
            chip.gpr[x as usize] = chip.gpr[x as usize] + nn as u8;
            println!("ADD V{:04X}", x);
        }
        0x8 => {
            match n {
                0 => {
                    chip.gpr[x as usize] = chip.gpr[y as usize];
                    println!("LD V{:04X} V{:04X}",x,y);
                }
                1 => {
                    chip.gpr[x as usize] = chip.gpr[x as usize] | chip.gpr[y as usize];
                    println!("OR V{:04X} V{:04X}",x,y);
                }
                2 => {
                    chip.gpr[x as usize] = chip.gpr[x as usize] & chip.gpr[y as usize];
                    println!("AND V{:04X} V{:04X}",x,y);
                }
                3 => {
                    chip.gpr[x as usize] = chip.gpr[x as usize] ^ chip.gpr[y as usize];
                    println!("XOR V{:04X} V{:04X}",x,y);
                }
                4 => {
                    let result : u16 = (chip.gpr[x as usize] + chip.gpr[y as usize]) as u16;
                    chip.gpr[x as usize] = (result & 0xFF) as u8;
                    chip.gpr[n as usize] = if result > 0xFF {1} else {0};
                    println!("ADD {} {}",chip.gpr[x as usize], chip.gpr[y as usize]);
                }
                5 => {
                    chip.gpr[0xF] = if chip.gpr[x as usize] >= chip.gpr[y as usize] {1} else {0};
                    chip.gpr[x as usize] = chip.gpr[x as usize] - chip.gpr[y as usize];
                    println!("SUB {} {}",chip.gpr[x as usize], chip.gpr[y as usize]);
                }
                6 => {
                    let lsb : u8  = chip.gpr[x as usize] & (0x0001);
                    if lsb == 0x0001 {
                        chip.gpr[n as usize] = 1;
                    } else {
                        chip.gpr[n as usize] = 0;
                    }
                    chip.gpr[x as usize] /= 2;
                    println!("SHR V{:04X} {{,V{:04X}}}",x,y);
                }
                7 => {
                    chip.gpr[0xF] = if chip.gpr[y as usize] >= chip.gpr[x as usize] {1} else {0};
                    chip.gpr[x as usize] = chip.gpr[y as usize] - chip.gpr[x as usize];
                }
                #[allow(non_snake_case)]
                _E => {
                    let lsb : u8  = chip.gpr[x as usize] & (0x0001);
                    if lsb == 0x0001 {
                        chip.gpr[n as usize] = 1;
                    } else {
                        chip.gpr[n as usize] = 0;
                    }
                    chip.gpr[x as usize] *= 2;
                    println!("SHL V{:04X} {{,V{:04X}}}",x,y);
                }
            }
        }
        0x9 => {    //Skip Conditional
            if chip.gpr[x as usize] != chip.gpr[y as usize] { chip.pc += 2; }
            println!("SNE V{} V{}",x,y);
        }
        0xA => {
            chip.index = nnn;
            println!("LD I {}",nnn)
        }
        0xB => {
            chip.pc = (chip.gpr[0] as u16) + nnn;
            println!("JP V0 {}",nnn);
        }
        0xC => {
            let rnd_byte = thread_rng().gen_range(0..=255);
            chip.gpr[x as usize] = (rnd_byte & nn) as u8;
            println!("RND V{}, {}",x,rnd_byte);
        }
        0xD => {

        }
        0xE => {
            match nn {
                0x9E => {
                    if let Some(&(_, chip_key, rl_key)) = chip.keys.iter().find(|&&(key, _, _)| key == chip.gpr[x as usize] as char) {
                        if rl_handle.is_key_down(rl_key) && chip_key == chip.gpr[x as usize] {
                            chip.pc += 2;
                            println!("SKP V{}", x);
                        }
                    }
                }
                0xA1 => {
                    if let Some(&(_, _, rl_key)) = chip.keys.iter().find(|&&(_, chip_key, _)| chip_key == chip.gpr[x as usize]) {
                        if rl_handle.is_key_up(rl_key) {
                            chip.pc += 2;
                            println!("SKNP V{}", x);
                        }
                    }
                }
                _ => {}
            }
        }
        0xF => {
            match nn {
                0x07 => {
                    chip.gpr[x as usize] = chip.d_timer;
                    println!("LD V{}, {}", chip.gpr[x as usize], chip.d_timer);
                }
                0x0A => {
                    if rl_handle.get_key_pressed() == Option::from(KEY_NULL) {
                        chip.pc -= 2;
                        println!("Fx0A - Polling");
                    } else {
                        if let Some(pressed_key) = rl_handle.get_key_pressed() {
                            if let Some(&(_, chip_key, _)) = chip.keys.iter().find(|&&(_, _, rl_key)| rl_key == pressed_key) {
                                chip.gpr[x as usize] = chip_key;
                            }
                        }
                    }
                    println!("LD V{} {}", x, n);
                }
                0x15 => {
                    chip.d_timer = chip.gpr[x as usize];
                    println!("LD {}, V{}", chip.d_timer, chip.gpr[x as usize]);
                }
                0x18 => {
                    chip.s_timer = chip.gpr[x as usize];
                    println!("LD {}, V{}",chip.s_timer ,chip.gpr[x as usize]);
                }
                0x1E => {
                    chip.index += chip.gpr[x as usize] as u16;
                    println!("ADD {}, V{}", chip.index, chip.gpr[x as usize]);
                }
                0x29 => {
                    chip.index = (chip.gpr[x as usize] as u16) * 5 + 80;
                    println!("LD {}, V{}", n, x);
                }
                0x33 => {

                }
                0x55 => {
                    for i in 0..chip.gpr.len() {
                        chip.memory[chip.index as usize + i] = chip.gpr[i];
                    }
                    println!("LD [{}], V{}", chip.index, x);
                }
                0x65 => {
                    for i in 0..chip.gpr.len() {
                        chip.gpr[i] = chip.memory[chip.index as usize + i];
                    }
                    println!("LD V{}, [{}]", x, chip.index);
                }
                _ => {}
            }
        }
        _ => {}
    }
    print!("\n");
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
    mem_dump(&chip.memory, 0);

    let (mut rl, thread) = init()
        .size(640,320)
        .title("Chip 8")
        .build();



    rl.set_target_fps(60);
    while !rl.window_should_close(){

        if chip.s_timer > 0 {
            if chip.s_timer == 1 {
                print!("\x07");
            }
            chip.s_timer -= 1;
        }

        execute(&mut chip, &mut rl); // Process a CPU cycle

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);
    }

}
