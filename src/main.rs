use core::panic;

use minifb::{Key, Scale, Window, WindowOptions};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

fn main() {
    let buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "chip8emu",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X16,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
