use crate::font::FONT_SET;
use std::time::Duration;

const CHIP8_OPCODE_SIZE :usize = 2;
const CHIP8_REG_V :usize = 16;
const CHIP8_STACK :usize = 16;
const CHIP8_RAM :usize = 4096;
pub const CHIP8_WIDTH: usize = 64;
pub const CHIP8_HEIGHT: usize = 32;

enum ProgramCounter {
    Next,
    Skip,
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
    reg_sp: usize,
    reg_dt: u8,
    timer_cycle: Duration,
    keypad: [bool; 16]
}

impl Processor {

    pub fn new() -> Self {

        let mut ram = [0; CHIP8_RAM];
        for (i, &byte) in FONT_SET.iter().enumerate() {
            ram[i] = byte;
        }

        Processor {
            ram,
            vram: [[0; CHIP8_WIDTH]; CHIP8_HEIGHT],
            vram_changed: false,
            reg_v: [0; CHIP8_REG_V],
            reg_pc: 0x200,
            reg_sp: 0,
            reg_i: 0,
            stack: [0; CHIP8_STACK],
            reg_dt: 0,
            timer_cycle: Duration::ZERO,
            keypad: [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false]
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

    pub fn tick(&mut self, delta: Duration, keypad:[bool; 16]) -> OutputState {
        
        self.keypad = keypad;
        self.vram_changed = false;
        self.update_delay_timer(delta);

        let opcode: u16 = self.read_opcode();
        let pc: ProgramCounter = self.run_opcode(opcode);

        match pc {
            ProgramCounter::Next => self.reg_pc += CHIP8_OPCODE_SIZE,
            ProgramCounter::Skip => self.reg_pc += CHIP8_OPCODE_SIZE * 2,
            ProgramCounter::Jump(addr) => self.reg_pc = addr,
        }
       
        OutputState {
            vram: &self.vram,
            vram_changed: self.vram_changed
        }
    }

    fn update_delay_timer(&mut self, delta: Duration) {

        let chip8_timer_period :Duration = Duration::from_secs_f32(1.0/60.0);
        self.timer_cycle += delta;

        if self.reg_dt > 0 && self.timer_cycle >= chip8_timer_period {
            self.reg_dt -= 1;
        }
    }

    fn read_opcode(&self) -> u16 {

        use std::io::Cursor;
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

        //println!("pc: {:x}, {:x}", self.reg_pc, opcode);

        return match hex_digits {
            (0x00, 0x00, 0x0e, 0x0e) => self.op_00ee(),
            (0x00, 0x00, 0x0e, 0x00) => self.op_00e0(),
            (0x01, _, _, _) => self.op_1nnn(addr),
            (0x02, _, _, _) => self.op_2nnn(addr),
            (0x03, _, _, _) => self.op_3xkk(vx, kk),
            (0x06, _, _, _) => self.op_6xkk(vx, kk),
            (0x07, _, _, _) => self.op_7xkk(vx, kk),
            (0x08, _, _, 0x0e) => self.op_8xye(vx, vy),
            (0x08, _, _, 0x06) => self.op_8xy6(vx, vy),
            (0x0a, _, _, _) => self.op_annn(addr),
            (0x0d, _, _, _) => self.op_dxyn(vx, vy, n),
            (0x0e, _, 0x0a, 0x01) => self.op_exa1(vx),
            (0x0c, _, _, _) => self.op_cxkk(vx, kk),
            (0x0f, _, 0x01, 0x05) => self.op_fn15(vx),
            (0x0f, _, 0x02, 0x09) => self.op_fx29(vx),
            (0x0f, _, 0x03, 0x03) => self.op_fx33(vx),
            (0x0f, _, 0x06, 0x05) => self.op_fx65(vx),
            (0x0f, _, 0x00, 0x07) => self.op_fx07(vx),

            _ => panic!("unexpected opcode {:#4X}", opcode)
        };
    }

    /*
     * JMP addr
     * Set the PC to address
     */
     fn op_1nnn(&mut self, addr:usize) -> ProgramCounter {
        ProgramCounter::Jump(addr)
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
     * RET
     * Return from a subroutine.
     */
     fn op_00ee(&mut self) -> ProgramCounter {
        self.reg_sp -= 1;
        let addr = self.stack[self.reg_sp] as usize;
        self.stack[self.reg_sp] = 0;

        ProgramCounter::Jump(addr)
    }

    /*
     * Set Vx = kk
     */
    fn op_6xkk(&mut self, vx:usize, kk:u8) -> ProgramCounter {
        self.reg_v[vx] = kk;
        ProgramCounter::Next
    }

    /*
     * Set I = addr
     */
    fn op_annn(&mut self, addr:usize) -> ProgramCounter {
        self.reg_i = addr;
        ProgramCounter::Next
    }

    /*
     * DRW Vx, Vy, nibble
     * Display n-byte sprite starting at memory location I at (Vx, Vy)
     */
    fn op_dxyn(&mut self, vx:usize, vy:usize, n:u8) -> ProgramCounter {

        self.reg_v[0xf] = 0;

        for byte in 0..n {
            let y : usize = ((self.reg_v[vy] + byte)).into();
            for bit in 0..8 {
                let x : usize = ((self.reg_v[vx] + bit)).into();

                if x < CHIP8_WIDTH && y < CHIP8_HEIGHT {
                    let color = (self.ram[self.reg_i + byte as usize] >> (7 - bit)) & 1;
                    self.reg_v[0xf] |= color & self.vram[y][x];
                    self.vram[y][x] ^= color;
                }
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

    /*
     * LD F, Vx
     * Set I = location of sprite for digit Vx.
     */
    fn op_fx29(&mut self, vx:usize) -> ProgramCounter {
        self.reg_i = (self.reg_v[vx] * 5).into();
        ProgramCounter::Next
    }
     
    /*
     * ADD Vx, byte
     * Set Vx = Vx + kk.
     */
    fn op_7xkk(&mut self, vx:usize, kk:u8) -> ProgramCounter {
        self.reg_v[vx] += kk;
        ProgramCounter::Next
    }

    /*
     * CLS
     * Clear the vram
     */
    fn op_00e0(&mut self) -> ProgramCounter {
        for y in 0..CHIP8_HEIGHT {
            for x in 00..CHIP8_HEIGHT {
                self.vram[y][x] = 0;
            }
        }

        self.vram_changed = true;

        ProgramCounter::Next
    }

    /*
     * LD DT, Vx
     * Set delay timer = Vx.
     */
    fn op_fn15(&mut self, vx: usize) -> ProgramCounter {
        self.reg_dt = self.reg_v[vx];
        ProgramCounter::Next
    }

    /*
     * LD Vx, DT
     * Set Vx = delay timer value.
     */
    fn op_fx07(&mut self, vx: usize) -> ProgramCounter {
        self.reg_v[vx] = self.reg_dt;
        ProgramCounter::Next
    }

    /*
     * SE Vx, byte
     * Skip next instruction if Vx = kk.
     */
    fn op_3xkk(&mut self, vx:usize, kk:u8) -> ProgramCounter {
        if self.reg_v[vx] == kk {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }
    
    /*
     * RND Vx, byte
     * Set Vx = random byte AND kk.
     */
    fn op_cxkk(&mut self, vx:usize, kk:u8) -> ProgramCounter {
        self.reg_v[vx] = rand::random::<u8>() & kk;
        ProgramCounter::Next
    }

    /*
     * SKNP Vx
     * Skip next instruction if key with the value of Vx is not pressed.
     */
    fn op_exa1(&mut self, vx:usize) -> ProgramCounter {
        if self.keypad[self.reg_v[vx] as usize] == false {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    /*
     *  SHL Vx {, Vy}
     *  Set Vx = Vx << 1 or Vy if > 0.
     */
    fn op_8xye(&mut self, vx:usize, vy:usize) -> ProgramCounter {

        self.reg_v[0xf] = (self.reg_v[vx] & 0b1000_0000) >> 7; 
        if vx == vy {
            self.reg_v[vx] = self.reg_v[vx] << 1
        } else {
            self.reg_v[vx] = self.reg_v[vx] << self.reg_v[vy]
        }

        ProgramCounter::Next
    }

    /*
     *  SHR Vx {, Vy}
     *  Set Vx = Vx >> 1 or Vy if > 0.
    */
    fn op_8xy6(&mut self, vx:usize, vy:usize) -> ProgramCounter {
        self.reg_v[0xf] = self.reg_v[vx] & 0b0000_0001; 
        if vx == vy {
            self.reg_v[vx] = self.reg_v[vx] >> 1
        } else {
            self.reg_v[vx] = self.reg_v[vx] >> self.reg_v[vy]
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
        assert_eq!(p.ram[0..80], FONT_SET);
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
    fn op_annn() {
        let mut p = Processor::new();
        p.op_annn(0x123);
        
        assert_eq!(p.reg_i, 0x123);
    }
    
    #[test]
    fn op_2nnn() {
        let mut p = Processor::new();
        let pc = p.op_2nnn(0x123);
        
        assert_eq!(p.stack[0], 0x202);
        assert_eq!(p.reg_sp, 1);
        assert!(matches!(pc, ProgramCounter::Jump(0x123)));
    }

    #[test]
    fn op_dxyn() {
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

    #[test]
    fn op_fx29() {
        let mut p = Processor::new();
        p.reg_v[3] = 5;
        p.op_fx29(3);

        assert_eq!(p.reg_i, 25);
    }
    
    #[test]
    fn op_7xkk() {
        let mut p = Processor::new();
        p.reg_v[3] = 5;
        p.op_7xkk(3, 15);

        assert_eq!(p.reg_v[3], 20);
    }

    #[test]
    fn op_00ee() {
        let mut p = Processor::new();
        p.op_2nnn(0x100);
        let pc = p.op_00ee();

        assert!(matches!(pc, ProgramCounter::Jump(0x202)));
        assert_eq!(p.reg_sp, 0);
    }

    #[test]
    fn op_00e0() {
        let mut p = Processor::new();

        p.vram[1][1] = 1;
        p.op_00e0();
        assert_eq!(p.vram[1][1] ,0);
        assert_eq!(p.vram_changed, true);
    }

    #[test]
    fn op_1nnn() {
        let mut p = Processor::new();
        let pc = p.op_1nnn(0x123);
        assert!(matches!(pc, ProgramCounter::Jump(0x123)));
    }

    #[test]
    fn op_fn15() {
        let mut p = Processor::new();
        p.reg_v[0x1] = 15;
        p.op_fn15(0x1);

        assert_eq!(p.reg_dt, 15);
    }

    #[test]
    fn op_fx07() {
        let mut p = Processor::new();
        p.reg_dt = 15;
        p.op_fx07(0x1);
        assert_eq!(p.reg_v[0x1], 15);
    }

    #[test]
    fn delay_timer() {
        let mut p = Processor::new();
        
        p.reg_dt = 100;
        for _ in 0..60 {
            p.update_delay_timer(Duration::from_secs_f32(1.0/60.0));
        }

        assert_eq!(p.timer_cycle.as_secs(), Duration::from_secs(1).as_secs());
        assert_eq!(p.reg_dt, 40);
    }

    #[test]
    fn op_3xkk() {
        let mut p = Processor::new();
        p.reg_v[0x1] = 15;

        let pc1 = p.op_3xkk(0x1, 10);
        let pc2 = p.op_3xkk(0x1, 15);
        let pc3 = p.op_3xkk(0x1, 20);

        assert!(matches!(pc1, ProgramCounter::Next));
        assert!(matches!(pc2, ProgramCounter::Skip));
        assert!(matches!(pc3, ProgramCounter::Next));
    }

    #[test]
    fn op_exa1() {
        let mut p = Processor::new();
        p.load(&[0x00, 0xe0]);

        let keymap = [ true, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false];
        p.tick(Duration::ZERO, keymap);
        p.reg_v[0x0] = 0;
        p.reg_v[0x1] = 1;

        let pc1 = p.op_exa1(0x0);
        let pc2 = p.op_exa1(0x1);

        assert!(matches!(pc1, ProgramCounter::Next));
        assert!(matches!(pc2, ProgramCounter::Skip));
    }

    #[test]
    fn op_8xye(){
        let mut p = Processor::new();

        p.reg_v[0] = 0b0000_0001;
        p.reg_v[1] = 4;
        p.op_8xye(0, 1);
        assert_eq!(p.reg_v[0], 0b0001_0000);

        p.reg_v[1] = 0b0000_0001;
        p.op_8xye(1, 1);
        assert_eq!(p.reg_v[1], 0b0000_0010);

        p.reg_v[0] = 0b0000_0001;
        p.op_8xye(0, 0);
        assert_eq!(p.reg_v[0], 0b0000_0010);

        p.reg_v[0] = 0b0000_0001;
        p.op_8xye(0, 0);
        assert_eq!(p.reg_v[0xf], 0);

        p.reg_v[0] = 0b1000_0000;
        p.op_8xye(0, 0);
        assert_eq!(p.reg_v[0xf], 1);
    }

    #[test]
    fn op_8xy6(){
        let mut p = Processor::new();

        p.reg_v[0] = 0b1000_0000;
        p.reg_v[1] = 4;
        p.op_8xy6(0, 1);
        assert_eq!(p.reg_v[0], 0b0000_1000);

        p.reg_v[1] = 0b1000_0000;
        p.op_8xy6(1, 1);
        assert_eq!(p.reg_v[1], 0b0100_0000);

        p.reg_v[0] = 0b1000_0000;
        p.op_8xy6(0, 0);
        assert_eq!(p.reg_v[0], 0b0100_0000);

        p.reg_v[0] = 0b1000_0000;
        p.op_8xy6(0, 0);
        assert_eq!(p.reg_v[0xf], 0);

        p.reg_v[0] = 0b0000_0001;
        p.op_8xy6(0, 0);
        assert_eq!(p.reg_v[0xf], 1);
    }
}