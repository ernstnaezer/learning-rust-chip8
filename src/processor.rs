use std::io::Cursor;

const CHIP8_OPCODE_SIZE :usize = 2;
const CHIP8_REG_V :usize = 16;
const CHIP8_STACK :usize = 16;
const CHIP8_RAM :usize = 4096;
pub const CHIP8_WIDTH: usize = 64;
pub const CHIP8_HEIGHT: usize = 32;

enum ProgramCounter {
    Next,
    Jump(usize)
}

pub struct OutputState<'a> {
    pub vram: &'a[[u8; CHIP8_WIDTH]; CHIP8_HEIGHT],
    pub vram_changed: bool
}

pub struct Processor {
    ram:    [u8; CHIP8_RAM],
    vram:   [[u8; CHIP8_WIDTH]; CHIP8_HEIGHT],
    vram_changed: bool,
    reg_v:  [u8; CHIP8_REG_V],
    stack:  [usize; CHIP8_STACK],
    reg_i:  usize,
    reg_pc: usize,
    reg_sp: usize
}

impl Processor {

    pub fn new() -> Self {

        Processor {            
            ram: [0; CHIP8_RAM],
            vram: [[0; CHIP8_WIDTH]; CHIP8_HEIGHT],
            vram_changed: false,
            reg_v: [0; CHIP8_REG_V],
            reg_pc: 0x200,
            reg_sp: 0,
            reg_i: 0,
            stack: [0; CHIP8_STACK],
        }
    }

    pub fn reset_pc(&mut self) {
        self.reg_pc = 0x200;
        self.reg_sp = 0;
    }

    pub fn load(&mut self, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            let addr = 0x200 + i;
            if addr < CHIP8_RAM {
                self.ram[0x200 + i] = byte;
            } else {
                break;
            }
        }
    }

    pub fn tick(&mut self) -> OutputState {
        
        self.vram_changed = false;

        let opcode = self.read_opcode();
        let pc = self.run_opcode(opcode);

        match pc {
            ProgramCounter::Next => self.reg_pc += CHIP8_OPCODE_SIZE,
            ProgramCounter::Jump(addr) => self.reg_pc = addr            
        }
       
        OutputState {
            vram: &self.vram,
            vram_changed: self.vram_changed
        }
    }

    fn read_opcode(&self) -> u16 {

        use std::io::SeekFrom;
        use std::io::Seek;
        use byteorder::{BigEndian, ReadBytesExt};

        let mut rdr = Cursor::new(self.ram);
        rdr.seek(SeekFrom::Start(self.reg_pc as u64)).unwrap();
        rdr.read_u16::<BigEndian>().unwrap()
    }

    fn run_opcode(&mut self, opcode:u16) -> ProgramCounter {

        // unpack the opcode into 4 bit hex digits (nibbles)
        let hex_digits = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );

        let kk = (opcode & 0x00FF) as u8;
        let vx = hex_digits.1 as usize;
        let vy = hex_digits.2 as usize;
        let n = hex_digits.3 as u8;
        let addr = (opcode & 0x0FFF) as usize;

        let r = match hex_digits {
            (0x06, _, _, _) => self.op_6xkk(vx, kk),
            (0x02, _, _, _) => self.op_2nnn(addr),
            (0x0a, _, _, _) => self.op_annn(addr),
            (0x0d, _, _, _) => self.op_dxyn(vx, vy, n),
            (0x0f, _, 0x03, 0x03) => self.op_fx33(vx),
            (0x0f, _, 0x06, 0x05) => self.op_fx65(vx),

            _ => panic!("unexpected opcode {:#4X}", opcode)
        };

        r
    }

    /*
     * Set Vx = kk
     */
    fn op_6xkk(&mut self, vx:usize, kk:u8) -> ProgramCounter {
        println!("LD V{:x}, {}", vx, kk);
        self.reg_v[vx] = kk;
        ProgramCounter::Next
    }

    /*
     * Set I = addr
     */
    fn op_annn(&mut self, addr:usize) -> ProgramCounter {
        println!("LD I, {}", addr);
        self.reg_i = addr;
        ProgramCounter::Next
    }

    /*
     * Call addr
     */
    fn op_2nnn(&mut self, addr:usize) -> ProgramCounter {
        self.stack[self.reg_sp] = self.reg_pc + CHIP8_OPCODE_SIZE;
        self.reg_sp += 1;
        ProgramCounter::Jump(addr)
    }

    /*
     * DRW Vx, Vy, nibble
     * Display n-byte sprite starting at memory location I at (Vx, Vy)
     */
    fn op_dxyn(&mut self, vx:usize, vy:usize, n:u8) -> ProgramCounter {

        self.reg_v[0xf] = 0;

        for byte in 0..n {
            let y : usize = ((self.reg_v[vy] + byte) % CHIP8_HEIGHT as u8).into();
            for bit in 0..8 {
                let x : usize = ((self.reg_v[vx] + bit) % CHIP8_WIDTH as u8).into();

                let color = (self.ram[self.reg_i + byte as usize] >> (7 - bit)) & 1;
                self.reg_v[0xf] |= color & self.vram[y][x];
                self.vram[y][x] ^= color;
            }
        }

        self.vram_changed = true;
        ProgramCounter::Next
    }

    /*
     * LD B, Vx
     * Store BCD representation of Vx in memory locations I, I+1, and I+2.
     */
    fn op_fx33(&mut self, vx:usize) -> ProgramCounter {
        let x = self.reg_v[vx];
        
        self.ram[self.reg_i] = x / 100;
        self.ram[self.reg_i + 1] = (x % 100) / 10;
        self.ram[self.reg_i + 2] = x % 10;

        ProgramCounter::Next
    }
    
    /*
     * LD Vx, [I]
     * Read registers V0 through Vx from memory starting at location I.
     */
    fn op_fx65(&mut self, vx:usize) -> ProgramCounter {

        for i in 0..vx + 1 {
            self.reg_v[i] = self.ram[self.reg_i + i];
        }

        ProgramCounter::Next
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_initial_state() {
        let p = Processor::new();
        
        assert_eq!(p.reg_pc, 0x200);
        assert_eq!(p.vram_changed, false);
    }

    #[test]
    fn test_load() {
        let data = [1, 2, 3, 4, 5];
        let mut p = Processor::new();
        p.load(&data);

        assert_eq!(p.ram[0x200..0x205], [1, 2, 3, 4, 5]);
    }

    #[test]
    fn op_6xkk() {
        let mut p = Processor::new();
        for n in 0..15 {
            p.op_6xkk(n as usize, n);
            
            assert_eq!(p.reg_v[n as usize], n);
        }
    }

    #[test]
    fn op_annn(){
        let mut p = Processor::new();
        p.op_annn(0x123);
        
        assert_eq!(p.reg_i, 0x123);
    }
    
    #[test]
    fn op_2nnn(){
        let mut p = Processor::new();
        let pc = p.op_2nnn(0x123);
        
        assert_eq!(p.stack[0], 0x202);
        assert_eq!(p.reg_sp, 1);
        assert!(matches!(pc, ProgramCounter::Jump(0x123)));
    }

    #[test]
    fn op_dxyn(){
        let mut p = Processor::new();
        let data = [0b10101010];
        p.load(&data);
        p.reg_v[0] = 10;
        p.reg_v[1] = 20;
        p.reg_i = 0x200;
        p.op_dxyn(0, 1, 1);
        
        assert_eq!(p.vram_changed, true);
        assert_eq!(p.vram[20][10..18], [1,0,1,0,1,0,1,0]);
    }

    #[test]
    fn op_fx33() {
        let mut p = Processor::new();
        p.reg_i = 100;
        p.reg_v[0] = 145;
        p.op_fx33(0);

        assert_eq!(p.ram[100], 1);
        assert_eq!(p.ram[101], 4);
        assert_eq!(p.ram[102], 5);
    }

    #[test]
    fn op_fx65() {
        let mut p = Processor::new();
        let data = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16];
        p.load(&data);
        p.reg_i = 0x200;
        p.op_fx65(0xf);

        assert_eq!(p.reg_v, [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]);
    }
}