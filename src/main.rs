use std::fs;
use std::thread;
use std::time::Duration;

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use rand::{self, Rng};

const MICROSEC_PER_SEC: u64 = (10 as u64).pow(6);
const FREQUENCY: u64 = 4000;
const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const MEM_SIZE: usize = 4096;
const MEM_START: usize = 0x200;
const STACK_SIZE: usize = 16;
const MINIFB_WHITE: u32 = 0x00FFFFFF;
const MINIFB_BLACK: u32 = 0x00000000;

fn read_program_into_memory(ram: &mut [u8; MEM_SIZE]) {
    let program_bytes = fs::read("IBM Logo.ch8").expect("Cannot read file");

    let mut ram_counter = MEM_START;
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
    let mut rng = rand::rng();

    let mut display_buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut ram: [u8; MEM_SIZE] = [0; MEM_SIZE];
    let mut program_counter: usize = MEM_START;
    let mut index_register: u16 = 0;
    let mut stack: [usize; STACK_SIZE] = [0; STACK_SIZE];
    let mut stack_ptr: usize = MEM_SIZE - 1;
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

    // window.set_target_fps(60);

    read_program_into_memory(&mut ram);

    // emulation loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if program_counter >= 4096 {
            break;
        }

        let cur_instruction_high = ram[program_counter];
        let cur_instruction_low = ram[program_counter + 1];

        let high_first_nibble = cur_instruction_high / 0b00010000;
        let high_second_nibble = (cur_instruction_high & 0b00001111) as u8;
        let low_first_nibble = cur_instruction_low / 0b00010000;
        let low_second_nibble = (cur_instruction_low & 0b00001111) as u8;

        let nnn =
            (high_second_nibble as usize) * 0b0000000100000000 + (cur_instruction_low as usize);
        let x = high_second_nibble as usize;
        let y = low_first_nibble as usize;
        // dbg!(program_counter, cur_instruction_high, high_second_nibble);

        match high_first_nibble {
            0x0 => match cur_instruction_low {
                0xE0 => {
                    // println!("00E0: cleared display");
                    display_buffer.fill(0);
                    window
                        .update_with_buffer(&display_buffer, WIDTH, HEIGHT)
                        .unwrap();
                    program_counter += 2;
                }
                0xEE => {
                    program_counter = stack[stack_ptr];
                    stack_ptr -= 1;
                }
                _ => {
                    program_counter += 2;
                }
            },
            0x1 => {
                let jmp_pos: usize = nnn;
                program_counter = jmp_pos;
            }
            0x2 => {
                stack[stack_ptr] = program_counter;
                stack_ptr += 1;
                program_counter = nnn as usize;
            }
            0x3 => {
                if var_registers[high_second_nibble as usize] == cur_instruction_low {
                    program_counter += 4;
                } else {
                    program_counter += 2;
                }
            }
            0x4 => {
                if var_registers[high_second_nibble as usize] != cur_instruction_low {
                    program_counter += 4;
                } else {
                    program_counter += 2;
                }
            }
            0x5 => {
                if var_registers[high_second_nibble as usize]
                    == var_registers[low_first_nibble as usize]
                {
                    program_counter += 4;
                } else {
                    program_counter += 2;
                }
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
            0x8 => match low_second_nibble {
                0x0 => {
                    var_registers[high_second_nibble as usize] =
                        var_registers[low_first_nibble as usize];
                    program_counter += 2;
                }
                0x1 => {
                    var_registers[x] = var_registers[x] | var_registers[y];
                    program_counter += 2;
                }
                0x2 => {
                    var_registers[x] = var_registers[x] & var_registers[y];
                    program_counter += 2;
                }
                0x3 => {
                    var_registers[x] = var_registers[x] ^ var_registers[y];
                    program_counter += 2;
                }
                0x4 => {
                    if var_registers[x] as u16 + var_registers[y] as u16 > 255 {
                        var_registers[0xF] = 1;
                    }
                    var_registers[x] = var_registers[x].wrapping_add(var_registers[y]);
                    program_counter += 2;
                }
                0x5 => {
                    var_registers[0xF] = 1;
                    if (var_registers[x] as i16) - (var_registers[y] as i16) < 0 {
                        var_registers[0xF] = 0;
                    }
                    var_registers[x] = var_registers[x].wrapping_sub(var_registers[y]);
                    program_counter += 2;
                }
                0x6 => {
                    // TODO: configurable X = Y behavior
                    var_registers[x] = var_registers[y];
                    var_registers[0xF] = var_registers[x] % 2;
                    var_registers[x] = var_registers[x] >> 1;
                    program_counter += 2;
                }
                0x7 => {
                    var_registers[0xF] = 1;
                    if (var_registers[y] as i16) - (var_registers[x] as i16) < 0 {
                        var_registers[0xF] = 0;
                    }
                    var_registers[x] = var_registers[y].wrapping_sub(var_registers[x]);
                    program_counter += 2;
                }
                0xE => {
                    // TODO: configurable X = Y behavior
                    var_registers[x] = var_registers[y];
                    var_registers[0xF] = var_registers[x] / 0b10000000;
                    var_registers[x] = var_registers[x] << 1;
                    program_counter += 2;
                }

                _ => {
                    panic!("Invalid instruction");
                }
            },
            0x9 => {
                if var_registers[high_second_nibble as usize]
                    != var_registers[low_first_nibble as usize]
                {
                    program_counter += 4;
                } else {
                    program_counter += 2;
                }
            }
            0xA => {
                index_register = nnn as u16;
                program_counter += 2;
                // println!("A: set index register to {:#X}", index_register);
            }
            0xB => {
                program_counter = nnn + var_registers[x] as usize;
            }
            0xC => {
                let nn = cur_instruction_low;
                let random_number: u8 = rng.random_range(0..=255);
                var_registers[x] = nn & random_number;
                program_counter += 2;
            }
            0xD => {
                var_registers[0xF] = 0;

                let x_pos = var_registers[high_second_nibble as usize] % (WIDTH as u8);
                let y_pos = var_registers[low_first_nibble as usize] % (HEIGHT as u8);

                let n = low_second_nibble;

                // dbg!(x_pos, y_pos);

                // println!("D: Draw at ({x_pos}, {y_pos}) with {n} lines");

                for down_offset in 0..n {
                    if y_pos + down_offset >= HEIGHT as u8 {
                        break;
                    }

                    let mut mem_buffer: u8 = ram[index_register as usize + down_offset as usize];
                    // dbg!(index_register, down_offset, mem_buffer);
                    // println!("index + down = {}", index_register as usize + down_offset as usize);

                    for right_offset in 0..8 {
                        if x_pos + right_offset >= WIDTH as u8 {
                            break;
                        }

                        let buffer_pos: usize = ((x_pos as usize)
                            + right_offset as usize
                            + (y_pos as usize + down_offset as usize) * WIDTH)
                            as usize;

                        if mem_buffer > 0b01111111 {
                            if display_buffer[buffer_pos] == MINIFB_WHITE {
                                var_registers[0xF] = 1;
                                display_buffer[buffer_pos] = MINIFB_BLACK;
                            } else {
                                display_buffer[buffer_pos] = MINIFB_WHITE;
                            }
                        }
                        mem_buffer = mem_buffer << 1;
                    }
                }

                window
                    .update_with_buffer(&display_buffer, WIDTH, HEIGHT)
                    .unwrap();

                program_counter += 2;
            }
            0xE => match cur_instruction_low {
                0x9E => {}
                0xA1 => {}
                _ => (),
            },
            _ => {
                program_counter += 2;
            }
        }

        window.update();
        if window.is_key_down(Key::C) {
            println!("Key C is pressed")
        };
        thread::sleep(Duration::from_micros(MICROSEC_PER_SEC / FREQUENCY));
    }
}
