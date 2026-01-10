use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Simple Window - Press ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )?;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Fill the buffer with a gradient pattern
        for i in 0..WIDTH * HEIGHT {
            let x = i % WIDTH;
            let y = i / WIDTH;
            let r = (x * 255 / WIDTH) as u32;
            let g = (y * 255 / HEIGHT) as u32;
            let b = 128u32;
            buffer[i] = (r << 16) | (g << 8) | b;
        }

        // Update the window with our buffer
        window.update_with_buffer(&buffer, WIDTH, HEIGHT)?;
    }

    Ok(())
}