use std::fs::File;
use std::io::Read;
use std::path::Path;

const FONTSET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Chip8 {
    memory: [u8; 4096],
    v: [u8; 16], // General-purpose registers V0 to VF
    i: u16,      // Index register
    pc: u16,     // Program counter
    sp: u8,      // Stack pointer
    stack: [u16; 16],
    delay_timer: u8,
    sound_timer: u8,
    gfx: [u8; 64 * 32], // Monochrome display
    keypad: [bool; 16], // Key state
}

impl Chip8{

    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            gfx: [0; 64 * 32],
            keypad: [false; 16],
        };
        chip8.memory[0x50..0x50 + FONTSET.len()].copy_from_slice(&FONTSET);
        chip8
    }

    pub fn load_rom<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let mut file: File = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let start= 0x200;
        let end= start + buffer.len();

        if end > self.memory.len(){
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "ROM is too large to fit in memory",
            ));
        }

        self.memory[start..end].copy_from_slice(&buffer);

        Ok(())
    }

    pub fn emulate_cycle(&mut self){
        let opcode = (self.memory[self.pc as usize] as u16) << 8
           | (self.memory[(self.pc + 1) as usize] as u16);

        println!("Executing opcode: {:#06X}", opcode);

        match opcode & 0xF000 {
            0x00E0 => {     //00E0 - CLS

            }
            0x00EE =>{      //00EE - RET

            }
            0x1000 => self.pc = opcode & 0x0FFF,
            0x2000 =>{      //2nnn - CALL addr
                if self.sp as usize >= self.stack.len() {
                    panic!("Stack overflow");
                }
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = addr;
            }
            0x3000 => {     //3xkk - SE Vx, byte
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let byte = (opcode & 0x00FF) as u8;
                if self.v[x] == byte {self.pc += 2;}
            }
            0x4000 => {     //4xkk - SNE Vx, byte
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let byte = (opcode & 0x00FF) as u8;
                if self.v[x] != byte {self.pc += 2;}
            }
            0x5000 => {     //5xy0 - SE Vx, Vy
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 8) as usize;
                if self.v[x] != self.v[y] {self.pc += 2;}
            }
            0x6000 => {     //6xkk - LD Vx, byte
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let byte = (opcode & 0x00FF) as u8;
                self.v[x] = byte;
                self.pc += 2;
            }
            0x7000 => {     //7xkk - ADD Vx, byte
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let byte = (opcode & 0x00FF) as u8;
                self.v[x] += byte
            }
            0x8000 => {     //8xy0 - LD Vx, Vy
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 8) as usize;
                self.v[x] = self.v[y];
            }
            0x8001 => {     //8xy1 - OR Vx, Vy
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 8) as usize;
                self.v[x] = (self.v[y] | self.v[x]);
            }
            0x8002 => {     //8xy2 - AND Vx, Vy
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 8) as usize;
                self.v[x] = (self.v[y] | self.v[x]);
            }
            0x8003 => {     //8xy3 - XOR Vx, Vy
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 8) as usize;
                self.v[x] = (self.v[y] ^ self.v[x]);
            }
            0x8004 => {     //8xy4 - ADD Vx, Vy
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                if(self.v[y]> (0xFF - self.v[x])){
                    self.v[0xF] = 1; //carry 
                } else {
                    self.v[0xF] = 0; 
                }
                self.v[x] += self.v[y];
            }
            0x8005 => {     //8xy5 - SUB Vx, Vy
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                if (self.v[x] > self.v[y]){
                    self.v[0xF] = 1;
                } else {
                    self.v[0xF] = 0; 
                }
                self.v[x] -= self.v[y]
            }
            0x9000 => {     //9xy0 - SNE Vx, Vy
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                if self.v[x] != self.v[y] {self.pc += 2;}
            }
            0xA000 => {     //Annn - LD I, addr
                let addr = opcode & 0x0FFF;
                self.i = addr;
                self.pc += 2;
            }
            0xB000 => {     //Bnnn - JP V0, addr
                let addr = opcode & 0x0FFF;
                self.pc = addr + self.v[0];
            }
            0xC000 => {
                
            }
            0xD000 => {
                
            }
            0xE000 => {
                
            }
            0xF000 => {
                
            }
            _ => {
                println!("Unknown opcode: {:#06X}", opcode);
                self.pc += 2;
            }
        }
    }



}

