mod chip8;
use chip8::Chip8;

fn main() {
    let mut chip8 = Chip8::new();

    let rom_path = "../data/PONG"; // Change this to your ROM path
    match chip8.load_rom(rom_path) {
        Ok(_) => println!("ROM loaded successfully."),
        Err(e) => eprintln!("Failed to load ROM: {}", e),
    }
    chip8.emulate_cycle();
}
