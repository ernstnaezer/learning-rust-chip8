use std::io::Cursor;

const CHIP8_REG_VX :usize = 16;
const CHIP8_RAM :usize = 4096;

pub struct Processor {
    ram: [u8; CHIP8_RAM],
    reg_vx:[u8; CHIP8_REG_VX],
    reg_i:u16,
    reg_pc:u16
}

impl Processor {

    pub fn new() -> Self {

        Processor {
            reg_vx: [0; CHIP8_REG_VX],
            ram: [0; CHIP8_RAM],
            reg_pc: 0x200,
            reg_i: 0
        }
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

    pub fn run_one_instruction(&mut self) {
        
        let opcode = self.read_opcode();
        self.run_opcode(opcode);

        self.reg_pc+=2;
    }

    fn read_opcode(&mut self) -> u16 {

        use std::io::SeekFrom;
        use std::io::Seek;
        use byteorder::{BigEndian, ReadBytesExt};

        let mut rdr = Cursor::new(self.ram);
        rdr.seek(SeekFrom::Start(self.reg_pc as u64)).unwrap();
        rdr.read_u16::<BigEndian>().unwrap()

        //(self.ram[self.reg_pc] as u16) << 8 | (self.ram[self.reg_pc + 1] as u16)
    }

    fn run_opcode(&mut self, opcode:u16) {

        // unpack the opcode into 4 bit hex digits (nibbles)
        let hex_digits = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );

        let kk = (opcode & 0x00FF) as u8;
        let x = hex_digits.1 as usize;
        let nnn = (opcode & 0x0FFF) as u16;

        match hex_digits {
            (0x06, _, _, _) => self.op_6xkk(x, kk),
            (0x0a, _, _, _) => self.op_annn(nnn),

            _ => panic!("unexpected opcode {:#4X}", opcode)
        }
    }

    fn set_vx(&mut self, x:usize, kk:u8) {
        // Vf should not be set, the register is used as flag by some instructions
        if x < 15 {
            self.reg_vx[x] = kk;
        }
    }

    fn op_6xkk(&mut self, x:usize, kk:u8) {
        println!("LD V{:x}, {:#X}", x, kk);
        self.set_vx(x, kk);
    }

    fn op_annn(&mut self, nnn:u16) {
        println!("LD I, {:#X}", nnn);
        self.reg_i = nnn;
    }
}
