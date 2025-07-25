# 🕹️ CHIP-8 Emulator using Rust

A simple, educational CHIP-8 emulator written in Rust using the [`minifb`](https://crates.io/crates/minifb) crate for graphics and input. This project is intended for learning purposes and aims to replicate the basic functionality of the original CHIP-8 virtual machine from the 1970s.

---

## 🚀 Features

- Opcode support for all official CHIP-8 instructions
- Keyboard input using standard PC keyboard mapping
- 64x32 monochrome display
- Built-in font set (0-F)
- Delay and sound timers
- Easy-to-read codebase for learning emulation and Rust

---

## 📦 Requirements

- [Rust](https://www.rust-lang.org/) (latest stable recommended)
- [Cargo](https://doc.rust-lang.org/cargo/)
---

## 🛠️ Building and Running

```bash
git clone https://github.com/yourusername/chip8-rust.git
cd chip8-rust
cargo build 
cargo run
