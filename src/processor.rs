use std::io::Cursor;

const CHIP8_REG_V :usize = 16;
const CHIP8_STACK :usize = 16;
const CHIP8_RAM :usize = 4096;
pub const CHIP8_WIDTH: usize = 64;
pub const CHIP8_HEIGHT: usize = 32;

enum OpcodeResult {
    Next,
    Call
}

pub struct OutputState<'a> {
    pub vram: &'a[[u8; CHIP8_WIDTH]; CHIP8_HEIGHT],
    pub vram_changed: bool
}

pub struct Processor {
    ram: [u8; CHIP8_RAM],
    vram: [[u8; CHIP8_WIDTH]; CHIP8_HEIGHT],
    vram_changed: bool,
    reg_v:[u8; CHIP8_REG_V],
    stack: :[u8; CHIP8_STACK]
    reg_i:usize,
    reg_pc:usize,
    reg_sp:usize,
}

impl Processor {

    pub fn new() -> Self {

        Processor {            
            ram: [0; CHIP8_RAM],
            vram: [[0; CHIP8_WIDTH]; CHIP8_HEIGHT],
            vram_changed: false

            reg_v: [0; CHIP8_REG_V],
            reg_pc: 0x200,
            reg_sp: 0xff,
            reg_i: 0,
            stack: [0; CHIP8_STACK],
            
        }
    }

    pub fn reset_pc(&mut self) {
        self.reg_pc = 0x200;
        self.reg_sp = 0xff;
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
        let r = self.run_opcode(opcode);

        r match {
            Next => self.reg_pc+=2,
            _ =>
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

    fn run_opcode(&mut self, opcode:u16) -> OpcodeResult {

        // unpack the opcode into 4 bit hex digits (nibbles)
        let hex_digits = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );

        let kk = (opcode & 0x00FF) as u8;
        let x = hex_digits.1 as usize;
        let y = hex_digits.2 as usize;
        let n = hex_digits.3 as usize;
        let addr = (opcode & 0x0FFF) as usize;

        match hex_digits {
            (0x06, _, _, _) => self.op_6xkk(x, kk),
            (0x02, _, _, _) => self.op_2nnn(addr),
            (0x0a, _, _, _) => self.op_annn(addr),
            (0x0d, _, _, _) => self.op_dxyn(x,y,n),

            _ => panic!("unexpected opcode {:#4X}", opcode)
        }
    }

    /*
     * Set Vx = kk
     */
    fn op_6xkk(&mut self, x:usize, kk:u8) {
        println!("LD V{:x}, {}", x, kk);
        self.reg_v[x] = kk;
    }

    /*
     * Set I = addr
     */
    fn op_annn(&mut self, addr:usize) {
        println!("LD I, {}", addr);
        self.reg_i = addr;
    }

    /*
     * Call addr
     */
    fn op_2nnn(&mut self, addr:usize) {
        self.reg_sp++;
        self.stack[self.reg_sp] = self.reg_pc;
        self.reg_pc = addr;
    }

    /*
     * DRW Vx, Vy, nibble
     * Display n-byte sprite starting at memory location I at (Vx, Vy)
     */
    fn op_dxyn(&mut self, vx:usize, vy:usize, n:usize) {

        self.reg_v[0xf] = 0;

        for byte in 0..n {
            let y = (self.reg_v[vy] as usize + byte) % CHIP8_HEIGHT;
            for bit in 0..8 {
                let x = (self.reg_v[vx] as usize + bit) % CHIP8_WIDTH;

                let color = (self.ram[self.reg_i + byte] >> (7 - bit)) & 1;
                self.reg_v[0xf] |= color & self.vram[y][x];
                self.vram[y][x] ^= color;
            }
        }

        self.vram_changed = true;
    }
}
