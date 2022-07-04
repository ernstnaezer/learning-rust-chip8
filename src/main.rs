mod drivers;
mod processor;

use std::env;

use drivers::{CartridgeDriver};
use processor::Processor;

fn main() {
    let rom_file_name = env::args().nth(1).unwrap();
    let cartridge = CartridgeDriver::new(rom_file_name);

    let mut processor = Processor::new();
    processor.load(&cartridge.rom);

    processor.run_one_instruction();
    processor.run_one_instruction();
    processor.run_one_instruction();
    processor.run_one_instruction();
    processor.run_one_instruction();
}
