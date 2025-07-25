mod chip8;
use chip8::Chip8;
use std::time::Duration;
use std::thread;

fn main() {
    let mut chip8 = Chip8::new();

    let rom_path = "./data/MERLIN"; 
    match chip8.load_rom(rom_path) {
        Ok(_) => println!("ROM loaded successfully."),
        Err(e) => eprintln!("Failed to load ROM: {}", e),
    }

    while chip8.window.is_open() && !chip8.window.is_key_down(minifb::Key::Escape) {
        chip8.update_keys();       
        chip8.emulate_cycle();     
        chip8.render();
            
        thread::sleep(Duration::from_millis(2)); 
    }
}
