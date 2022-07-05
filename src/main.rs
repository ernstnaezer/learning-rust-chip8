mod drivers;
mod processor;

use std::thread;
use std::time::Duration;
use std::env;

use drivers::{DisplayDriver, CartridgeDriver};
use processor::Processor;

fn main() {
    let rom_file_name = env::args().nth(1).unwrap();
    let cartridge = CartridgeDriver::new(rom_file_name);

    let sdl_context = sdl2::init().unwrap();

    let mut display = DisplayDriver::new(&sdl_context);
    let mut processor = Processor::new();
    processor.load(&cartridge.rom);

    let sleep_duration = Duration::from_millis(2);

    let mut event_pump = sdl_context.event_pump().unwrap();

    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }

            processor.reset_pc();
            for _ in 0..12 {
                let state = processor.tick();
                if state.vram_changed {
                    display.draw(&state.vram);
                }
            }

            thread::sleep(sleep_duration);
        }
    }
    
}
