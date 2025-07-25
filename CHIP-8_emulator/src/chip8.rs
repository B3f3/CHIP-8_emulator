use std::fs::File;
use std::io::Read;
use std::path::Path;
use minifb::{Window, WindowOptions, Key};

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
    pub pc: u16,     // Program counter
    sp: u8,      // Stack pointer
    stack: [u16; 16],
    delay_timer: u8,
    sound_timer: u8,
    gfx: [u8; 64 * 32], // Monochrome display
    keypad: [bool; 16], // Key state
    pub window: Window,
    buffer: Vec<u32>, // Pixel buffer (64x32)
}

impl Chip8{

    pub fn new() -> Self {
        let window = Window::new(
            "CHIP-8 Emulator",
            64 * 4,  // Width (scaled 4x)
            32 * 4,  // Height (scaled 4x)
            WindowOptions {
                scale: minifb::Scale::X4,
                ..WindowOptions::default()
            },
        ).expect("Failed to create window");

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
            window,
            buffer: vec![0; 64 * 32], // Black screen initially
        };
        chip8.memory[0x50..0x50 + FONTSET.len()].copy_from_slice(&FONTSET);
        println!("Fontset loaded at 0x50: {:02X?}", &chip8.memory[0x50..0x55]);
        chip8
    }

    pub fn update_keys(&mut self) {
        use minifb::Key;
        
        // CHIP-8 Keypad       Keyboard
        self.keypad = [
            self.window.is_key_down(Key::X),    // 0
            self.window.is_key_down(Key::Key1), // 1
            self.window.is_key_down(Key::Key2), // 2
            self.window.is_key_down(Key::Key3), // 3
            self.window.is_key_down(Key::Q),    // 4
            self.window.is_key_down(Key::W),    // 5
            self.window.is_key_down(Key::E),    // 6
            self.window.is_key_down(Key::A),    // 7
            self.window.is_key_down(Key::S),    // 8
            self.window.is_key_down(Key::D),    // 9
            self.window.is_key_down(Key::Z),    // A
            self.window.is_key_down(Key::C),    // B
            self.window.is_key_down(Key::Key4), // C
            self.window.is_key_down(Key::R),    // D
            self.window.is_key_down(Key::F),    // E
            self.window.is_key_down(Key::V),    // F
        ];
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

    fn op_00e0(&mut self) {
        self.gfx = [0; 64 * 32];
        self.pc += 2;
    }

    fn op_00ee(&mut self) {
        if self.sp == 0 {
            panic!("Stack underflow");
        }
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
        self.pc += 2;
    }

    fn op_2nnn(&mut self, addr: u16) {
        if self.sp as usize >= self.stack.len() {
            panic!("Stack overflow");
        }
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = addr;
    }

    fn op_dxyn(&mut self, x: usize, y: usize, n: u8) {
        self.v[0xF] = 0;
        for byte in 0..n {
            let y = (self.v[y] as usize + byte as usize) % 32;
            let sprite_byte = self.memory[self.i as usize + byte as usize];
            
            for bit in 0..8 {
                let x = (self.v[x] as usize + bit) % 64;
                let pixel = (sprite_byte >> (7 - bit)) & 1;
                
                if pixel == 1 {
                    let idx = y * 64 + x;
                    if self.gfx[idx] == 1 {
                        self.v[0xF] = 1;
                    }
                    self.gfx[idx] ^= 1;
                }
            }
        }
        self.pc += 2;
    }

    /// Returns the first pressed key (0x0-0xF) if any, or None
    fn get_pressed_key(&self) -> Option<usize> {
        self.keypad.iter()
            .enumerate()
            .find_map(|(i, &pressed)| if pressed { Some(i) } else { None })
    }

    fn fetch_opcode(&self) -> u16 {
        ((self.memory[self.pc as usize] as u16) << 8) 
            | (self.memory[(self.pc + 1) as usize] as u16)
    }

    pub fn emulate_cycle(&mut self){
        let opcode = self.fetch_opcode();

        println!("Executing opcode: {:#06X}", opcode);
        let x = ((opcode & 0x0F00) >> 8) as usize;
        println!("SE V{:X}, {:02X} (V[{}]={:02X})", 
        (opcode & 0x0F00) >> 8,
        opcode & 0x00FF,
        x,
        self.v[x]);

        match opcode & 0xF000 {         //all opcode cases
            0x0000 => match opcode {
                0x00E0 => self.op_00e0(),
                0x00EE => self.op_00ee(),
                _ => self.pc += 2, // SYS addr - ignore
            }
            0x1000 => self.pc = opcode & 0x0FFF,
            0x2000 => self.op_2nnn(opcode & 0x0FFF),
            0x3000 => {     //3xkk - SE Vx, byte    
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let byte = (opcode & 0x00FF) as u8;
                if self.v[x] == byte {self.pc += 2;}
                self.pc += 2;
            }
            0x4000 => {     //4xkk - SNE Vx, byte   
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let byte = (opcode & 0x00FF) as u8;
                if self.v[x] != byte {self.pc += 2;}
                self.pc += 2;
            }
            0x5000 => {     //5xy0 - SE Vx, Vy  
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 8) as usize;
                if self.v[x] != self.v[y] {self.pc += 2;}
                self.pc += 2;
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
                self.v[x] += byte;
                self.pc += 2;
            }
            0x8000 => match opcode & 0xF00F {       //800F cases    
                0x8000 => {     //8xy0 - LD Vx, Vy  
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 8) as usize;
                    self.v[x] = self.v[y];
                    self.pc += 2;
                }
                0x8001 => {     //8xy1 - OR Vx, Vy  
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 8) as usize;
                    self.v[x] |= self.v[y];
                    self.pc += 2;
                }
                0x8002 => {     //8xy2 - AND Vx, Vy 
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 8) as usize;
                    self.v[x] &= self.v[y];
                    self.pc += 2;
                }
                0x8003 => {     //8xy3 - XOR Vx, Vy 
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 8) as usize;
                    self.v[x] ^= self.v[y];
                    self.pc += 2;
                }
                0x8004 => {     //8xy4 - ADD Vx, Vy 
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 8) as usize;
                    let sum = (self.v[x] as u16) + (self.v[y] as u16);
                    self.v[0xF] = if sum > 0xFF { 1 } else { 0 };
                    self.v[x] = sum as u8;
                    self.pc += 2;
                }
                0x8005 => {     //8xy5 - SUB Vx, Vy 
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 8) as usize;
                    self.v[0xF] = if self.v[x] >= self.v[y] { 1 } else { 0 };
                    self.v[x] = self.v[x].wrapping_sub(self.v[y]);
                    self.pc += 2;
                }
                0x8006 => {     //8xy6 - SHR Vx {, Vy}  
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.v[0xF] = self.v[x] & 0x1;  // Store LSB in VF
                    self.v[x] >>= 1;
                    self.pc += 2;
                }
                0x8007 => {     //8xy7 - SUBN Vx, Vy    
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 8) as usize;
                    self.v[0xF] = if self.v[y] >= self.v[x] { 1 } else { 0 };
                    self.v[x] = self.v[y].wrapping_sub(self.v[x]);
                    self.pc += 2;
                }
                0x800E => {     //8xyE - SHL Vx {, Vy}  
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.v[0xF] = (self.v[x] >> 7) & 0x1;  // Store MSB in VF
                    self.v[x] <<= 1;
                    self.pc += 2;
                }
                _ => self.pc += 2
            }
            0x9000 => {     //9xy0 - SNE Vx, Vy
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 8) as usize;
                if self.v[x] != self.v[y] {self.pc += 2;}
                self.pc += 2;
            }
            0xA000 => {     //Annn - LD I, addr
                let addr = opcode & 0x0FFF;
                self.i = addr;
                self.pc += 2;
            }
            0xB000 => {     //Bnnn - JP V0, addr
                let base_addr = opcode & 0x0FFF;
                let offset = self.v[0x0] as u16;  // Explicitly use V0
                self.pc = base_addr.wrapping_add(offset);
            }
            0xC000 => {     //Cxkk - RND Vx, byte
                let x = ((opcode & 0x0F00) >> 8) as usize;  
                let byte = (opcode & 0x00FF) as u8;          
                let random_byte: u8 = rand::random();  
                self.v[x] = random_byte & byte;
                self.pc += 2; 
            }
            0xD000 => {     //Dxyn - DRW Vx, Vy, nibble
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 8) as usize;
                let n = (opcode & 0x000F) as u8;
                self.op_dxyn(x, y, n);
            }
            0xE000 => match opcode & 0xF0FF{        //E0FF cases    
                0xE09E => {     //Ex9E - SKP Vx
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let key = self.v[x] as usize;
                    if key < 16 && self.keypad[key] {
                        self.pc += 2;
                    }
                    self.pc += 2; 
                }
                0xE0A1 => {     //ExA1 - SKNP Vx
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let key = self.v[x] as usize;
                    if key >= 16 || !self.keypad[key] {
                        self.pc += 2; 
                    }
                    self.pc += 2; 
                }
                _ => ()
            }
            0xF000 => match opcode & 0xF0FF{        //F0FF cases
                0xF007 =>{      //Fx07 - LD Vx, DT
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.v[x] = self.delay_timer;
                    self.pc += 2;
                }
                0xF00A =>{      //Fx0A - LD Vx, K
                        let x = ((opcode & 0x0F00) >> 8) as usize;
                        if let Some(key) = self.get_pressed_key() {
                            // Key was pressed - store it and advance
                            self.v[x] = key as u8;
                            self.pc += 2;
                        } else {
                            // No key pressed - stay on this opcode (don't increment PC)
                            // This effectively pauses execution until a key is pressed
                            return;
                        }
                }
                0xF015 =>{      //Fx15 - LD DT, Vx
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.delay_timer = self.v[x];
                    self.pc += 2;
                }
                0xF018 =>{      //Fx18 - LD ST, Vx
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.sound_timer = self.v[x];
                    self.pc += 2;
                }
                0xF01E =>{      //Fx1E - ADD I, Vx
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.i += self.v[x] as u16;
                    self.pc += 2;
                }
                0xF029 =>{      //Fx29 - LD F, Vx
                        let x = ((opcode & 0x0F00) >> 8) as usize;
                        let digit = self.v[x] & 0x0F; 
                        self.i = 0x50 + (digit as u16 * 5);
                        self.pc += 2;
                }
                0xF033 =>{      //Fx33 - LD B, Vx
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let vx = self.v[x];
                    self.memory[self.i as usize] = vx / 100;
                    self.memory[self.i as usize + 1] = (vx / 10) % 10;
                    self.memory[self.i as usize + 2] = vx % 10;
                    self.pc += 2;
                }   
                0xF055 =>{      //Fx55 - LD [I], Vx
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    for i in 0..=x {
                        self.memory[self.i as usize + i] = self.v[i];
                    }
                    self.pc += 2;
                }
                0xF065 =>{      //Fx65 - LD Vx, [I]
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    for i in 0..=x {
                        self.v[i] = self.memory[self.i as usize + i];
                    }
                    self.pc += 2;
                }
                _ => ()
            }
            _ => {          //unknows error
                println!("Unknown opcode: {:#06X}", opcode);
                self.pc += 2;
            }
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

    }

    pub fn render(&mut self) {
        // Convert 1-bit gfx to 32-bit RGB
        for (i, &pixel) in self.gfx.iter().enumerate() {
            self.buffer[i] = if pixel == 1 { 
                0xFFFFFF // White
            } else { 
                0x000000 // Black 
            };
        }

        self.window
            .update_with_buffer(&self.buffer, 64, 32)
            .unwrap();
    }

}

