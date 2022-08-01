mod drivers;
mod processor;
mod font;

use std::thread;
use std::time::Duration;
use std::env;
use std::time::Instant;

use drivers::{DisplayDriver, CartridgeDriver, InputDriver};
use processor::Processor;

fn main() -> ! {
    let rom_file_name = env::args().nth(1).unwrap();
    let cartridge = CartridgeDriver::new(rom_file_name);

    let sdl_context = sdl2::init().unwrap();

    let mut display = DisplayDriver::new(&sdl_context);
    let mut input: InputDriver = InputDriver::new(&sdl_context);
    let mut processor = Processor::new();
    processor.load(&cartridge.rom);

    let sleep_duration = Duration::from_millis(2);

    let mut start = Instant::now();

    'running: loop {

        let delta = start.elapsed();

        let keymap = input.update().unwrap();
        let state = processor.tick(delta, keymap);

        if state.vram_changed {
            display.draw(&state.vram);
        }

        start = Instant::now();
        thread::sleep(sleep_duration);
    }
    
}
