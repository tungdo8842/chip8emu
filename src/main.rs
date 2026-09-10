use std::fs;

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const MEM_SIZE: usize = 4096;
const MINIFB_WHITE: u32 = 0x00FFFFFF;
const MINIFB_BLACK: u32 = 0x00000000;

fn read_program_into_memory(ram: &mut [u8; MEM_SIZE]) {
    let program_bytes = fs::read("IBM Logo.ch8").expect("Cannot read file");

    let mut ram_counter = 0x200;
    for byte in program_bytes {
        // println!("byte content is {}", byte);
        ram[ram_counter] = byte;
        ram_counter += 1;
        if ram_counter >= MEM_SIZE {
            break;
        }
    }
}

fn main() {
    let mut display_buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut ram: [u8; MEM_SIZE] = [0; MEM_SIZE];
    let mut program_counter: usize = 0x200;
    let mut index_register: u16 = 0;
    // let stack: u16 = 4096;
    let mut var_registers: [u8; 16] = [0; 16];
    // let delay_timer: u8 = 0;
    // let sound_timer: u8 = 0;

    let mut window = Window::new(
        "chip8emu",
        WIDTH,
        HEIGHT * 2,
        WindowOptions {
            scale_mode: ScaleMode::Stretch,
            scale: Scale::X16,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(60);

    read_program_into_memory(&mut ram);

    // emulation loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if program_counter >= 4096 {
            break;
        }

        let cur_instruction_high = ram[program_counter];
        let cur_instruction_low = ram[program_counter + 1];

        let high_first_nibble = cur_instruction_high / 0b1111;
        let high_second_nibble = (cur_instruction_high & 0b00001111) as u8;
        let low_first_nibble = cur_instruction_low / 0b1111;
        let low_second_nibble = (cur_instruction_low & 0b00001111) as u8;

        // dbg!(program_counter, cur_instruction_high, high_second_nibble);

        match high_first_nibble {
            0x0 => match cur_instruction_low {
                0xE0 => {
                    println!("E: cleared display");
                    display_buffer.fill(0);
                    window
                        .update_with_buffer(&display_buffer, WIDTH, HEIGHT)
                        .unwrap();
                    program_counter += 2;
                }
                _ => {
                    program_counter += 2;
                }
            },
            0x1 => {
                let jmp_pos: usize =
                    (high_second_nibble as usize) * 0b00010000 + (cur_instruction_low as usize);
                program_counter = jmp_pos;
                println!("1: jump to {jmp_pos}");
            }
            0x6 => {
                let register_index = high_second_nibble as usize;
                var_registers[register_index] = cur_instruction_low;
                program_counter += 2;
            }
            0x7 => {
                let register_index = high_second_nibble as usize;
                var_registers[register_index] += cur_instruction_low;
                program_counter += 2;
            }
            0xA => {
                index_register =
                    (high_second_nibble as u16) * 0b00010000 + (cur_instruction_low as u16);
                program_counter += 2
            }
            0xD => {
                let x_pos = var_registers[high_second_nibble as usize] % (WIDTH as u8);
                let y_pos = var_registers[low_first_nibble as usize] % (HEIGHT as u8);

                let buffer_pos: usize = ((x_pos as usize) + (y_pos as usize) * WIDTH) as usize;
                display_buffer[buffer_pos] = 0x00FFFFFF;

                window
                    .update_with_buffer(&display_buffer, WIDTH, HEIGHT)
                    .unwrap();
                println!("D: draw at {x_pos}, {y_pos} with value {low_second_nibble}");
                program_counter += 2;
            }
            _ => {
                program_counter += 2;
            }
        }
    }
}
