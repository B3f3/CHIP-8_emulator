mod chip8;
use chip8::Chip8;

fn main() {
    let mut chip8 = Chip8::new();

    let rom_path = "./data/PONG"; // Change this to your ROM path
    match chip8.load_rom(rom_path) {
        Ok(_) => println!("ROM loaded successfully."),
        Err(e) => eprintln!("Failed to load ROM: {}", e),
    }

    while chip8.window.is_open() {
        // Handle input
        chip8.update_keys();
        
        // Emulate cycle
        chip8.emulate_cycle();
        
        // Render every frame (60Hz)
        if chip8.pc % 10 == 0 { // Adjust based on your clock speed
            chip8.render();
        }
        
        // Limit speed
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}
