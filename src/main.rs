use std::fs;
use std::thread;
use std::time::Duration;

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use rand::{self, Rng};

const NUM_KEYS: usize = 16;
const MICROSEC_PER_SEC: u32 = (10 as u32).pow(6);
const FREQUENCY: u32 = 4000;
const MICROSEC_PER_CLOCK: u32 = MICROSEC_PER_SEC / FREQUENCY;
const TIMING_HZ: u32 = 60;
const MICROSEC_PER_TIMING_TICK: u32 = MICROSEC_PER_SEC / TIMING_HZ;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const MINIFB_WHITE: u32 = 0x00FFFFFF;
const MINIFB_BLACK: u32 = 0x00000000;

const MEM_SIZE: usize = 4096;
const MEM_START: usize = 0x200;
const STACK_SIZE: usize = 16;
const FONT_START: usize = 0x50;
const FONT_SIZE: usize = 5;

fn font_init(ram: &mut [u8; MEM_SIZE]) {
    let start_pos: usize = FONT_START;
    let data: [u8; FONT_SIZE * NUM_KEYS] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
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
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];

    for i in 0..data.len() {
        ram[start_pos + i] = data[i];
    }
}

fn read_program_into_memory(ram: &mut [u8; MEM_SIZE]) {
    let program_bytes = fs::read("3-corax+.ch8").expect("Cannot read file");

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

fn get_current_keypresses(window: &Window) -> Vec<bool> {
    let mut keys_vec = vec![false; NUM_KEYS];
    for key in window.get_keys() {
        match key {
            Key::Key1 => keys_vec[0] = true,
            Key::Key2 => keys_vec[1] = true,
            Key::Key3 => keys_vec[2] = true,
            Key::Key4 => keys_vec[3] = true,
            Key::Q => keys_vec[4] = true,
            Key::W => keys_vec[5] = true,
            Key::E => keys_vec[6] = true,
            Key::R => keys_vec[7] = true,
            Key::A => keys_vec[8] = true,
            Key::S => keys_vec[9] = true,
            Key::D => keys_vec[10] = true,
            Key::F => keys_vec[11] = true,
            Key::Z => keys_vec[12] = true,
            Key::X => keys_vec[13] = true,
            Key::C => keys_vec[14] = true,
            Key::V => keys_vec[15] = true,
            _ => (),
        }
    }
    return keys_vec;
}

fn main() {
    let mut rng = rand::rng();

    let mut display_buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut ram: [u8; MEM_SIZE] = [0; MEM_SIZE];
    let mut program_counter: usize = MEM_START;
    let mut index_register: u16 = 0;
    let mut stack: [usize; STACK_SIZE] = [0; STACK_SIZE];
    let mut stack_ptr: usize = 0;
    let mut var_registers: [u8; 16] = [0; 16];

    let mut timing_usec: u32 = 0;
    let mut delay_timer: u8 = 0;
    let mut sound_timer: u8 = 0;

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

    font_init(&mut ram);
    read_program_into_memory(&mut ram);

    // emulation loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if program_counter >= (MEM_SIZE - MEM_START - 1) {
            break;
        }

        let current_keypresses: Vec<bool> = get_current_keypresses(&window);

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
                    program_counter = stack[stack_ptr - 1];
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
                program_counter = nnn;
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
                var_registers[x] = cur_instruction_low;
                program_counter += 2;
            }
            0x7 => {
                var_registers[x] = var_registers[x].wrapping_add(cur_instruction_low);
                program_counter += 2;
            }
            0x8 => match low_second_nibble {
                0x0 => {
                    var_registers[x] = var_registers[y];
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

                let x_pos = var_registers[x] % (WIDTH as u8);
                let y_pos = var_registers[y] % (HEIGHT as u8);

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
                0x9E => {
                    if current_keypresses[x] == true {
                        program_counter += 4;
                    } else {
                        program_counter += 2;
                    }
                }
                0xA1 => {
                    if current_keypresses[x] == false {
                        program_counter += 4;
                    } else {
                        program_counter += 2;
                    }
                }
                _ => (),
            },
            0xF => match cur_instruction_low {
                0x07 => {
                    var_registers[x] = delay_timer;
                    program_counter += 2;
                }
                0x0A => {
                    let keys = get_current_keypresses(&window);
                    for (index, key) in keys.iter().enumerate() {
                        // only go to next instruction if a key is pressed
                        if *key == true {
                            var_registers[x] = index as u8;
                            program_counter += 2;
                            break;
                        }
                    }
                }
                0x15 => {
                    delay_timer = var_registers[x];
                    program_counter += 2;
                }
                0x18 => {
                    sound_timer = var_registers[x];
                    program_counter += 2;
                }
                0x1E => {
                    index_register += var_registers[x] as u16;
                    program_counter += 2;
                    // TODO: Amiga VF behavior
                }
                0x29 => {
                    index_register = FONT_START as u16 + var_registers[x] as u16 * FONT_SIZE as u16;
                    program_counter += 2;
                }
                0x33 => {
                    let hundreds = var_registers[x] / 100; // u8, only go up to 255
                    let tens = var_registers[x] % 100 / 10;
                    let ones = var_registers[x] % 10;

                    ram[index_register as usize] = hundreds;
                    ram[index_register as usize + 1] = tens;
                    ram[index_register as usize + 2] = ones;

                    program_counter += 2;
                }
                // TODO: custom behivior toggle
                0x55 => {
                    for i in 0..=var_registers[x] {
                        ram[index_register as usize + i as usize] = var_registers[i as usize];
                    }
                    program_counter += 2;
                }
                0x66 => {
                    for i in 0..=var_registers[x] {
                        var_registers[i as usize] = ram[index_register as usize + i as usize];
                    }
                    program_counter += 2;
                }
                _ => (),
            },
            _ => {
                program_counter += 2;
            }
        }

        // update screen, wait, and update timers
        window.update();
        thread::sleep(Duration::from_micros(MICROSEC_PER_CLOCK as u64));
        timing_usec = (timing_usec + MICROSEC_PER_CLOCK) % MICROSEC_PER_SEC;
        if timing_usec % MICROSEC_PER_TIMING_TICK < MICROSEC_PER_CLOCK {
            if delay_timer > 0 {
                delay_timer -= 1
            };
            if sound_timer > 0 {
                sound_timer -= 1
            };
        }
    }
}
