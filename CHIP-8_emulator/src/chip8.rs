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
            0x00E0 => {

            }
            0x00EE =>{

            }
            0x1000 => {
                let addr = opcode & 0x0FFF;
                self.pc = addr;
            }
            0x2000 =>{
                let addr = opcode & 0x0FFF;
                self.sp += 1;
                self.stack[self.sp] == self.pc;
                self.pc = addr;
            }
            0x6000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let byte = (opcode & 0x00FF) as u8;
                self.v[x] = byte;
                self.pc += 2;
            }
            0xA000 => {
                let addr = opcode & 0x0FFF;
                self.i = addr;
                self.pc += 2;
            }

            _ => {
                println!("Unknown opcode: {:#06X}", opcode);
                self.pc += 2;
            }
        }
    }



}

