extern crate bit_vec;
extern crate minifb;

use std::process;
use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use bit_vec::BitVec;
use minifb::{Key, Scale, Window, WindowOptions};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

struct Chip8 {
    memory: [u8; 4096],
    v: [u8; 16],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: i8,
    opcode: u16,
    gfx: [u8; 8 * 32],
    pc: u16,
    i: u16,
}

impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {
            delay_timer: 0,
            sound_timer: 0,
            memory: [0; 4096],
            opcode: 0,
            sp: -1,
            gfx: [0x000; WIDTH / 8 * HEIGHT],
            v: [0; 16],
            stack: [0; 16],
            pc: 0x200,
            i: 0x0000,
        }
    }

    fn init_mem(&mut self) {
        let interp_mem = &mut self.memory[0..200];
        let start = 0x000;
        let sprite_ht = 5;
        // 0
        interp_mem[start + 0 * sprite_ht + 0] = 0xF0;
        interp_mem[start + 0 * sprite_ht + 1] = 0x90;
        interp_mem[start + 0 * sprite_ht + 2] = 0x90;
        interp_mem[start + 0 * sprite_ht + 3] = 0x90;
        interp_mem[start + 0 * sprite_ht + 4] = 0xF0;

        // 1
        interp_mem[start + 1 * sprite_ht + 0] = 0x20;
        interp_mem[start + 1 * sprite_ht + 1] = 0x60;
        interp_mem[start + 1 * sprite_ht + 2] = 0x20;
        interp_mem[start + 1 * sprite_ht + 3] = 0x20;
        interp_mem[start + 1 * sprite_ht + 4] = 0x70;

        // 2
        interp_mem[start + 2 * sprite_ht + 0] = 0xF0;
        interp_mem[start + 2 * sprite_ht + 1] = 0x10;
        interp_mem[start + 2 * sprite_ht + 2] = 0xF0;
        interp_mem[start + 2 * sprite_ht + 3] = 0x80;
        interp_mem[start + 2 * sprite_ht + 4] = 0xF0;

        // 3
        interp_mem[start + 3 * sprite_ht + 0] = 0xF0;
        interp_mem[start + 3 * sprite_ht + 1] = 0x10;
        interp_mem[start + 3 * sprite_ht + 2] = 0xF0;
        interp_mem[start + 3 * sprite_ht + 3] = 0x10;
        interp_mem[start + 3 * sprite_ht + 4] = 0xF0;
    }

    fn run(&mut self) {
        match self.opcode & 0xF000 {
            0x0000 => {
                if self.opcode == 0x00E0 {
                    self.gfx = [0; 8 * 32];
                    self.pc += 0x0002;
                } else if self.opcode == 0x00EE {
                    self.pc = self.stack[self.sp as usize];
                    self.sp -= 1;
                }
            }
            0x1000 => {
                self.pc = self.opcode & 0x0FFF;
            }
            0x2000 => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = self.opcode & 0x0FFF;
            }
            0x3000 => {
                if self.v[(self.opcode & 0x0F00) as usize >> 8] == (self.opcode & 0x00FF) as u8 {
                    self.pc += 0x0002;
                }
                self.pc += 0x0002;
            }
            0x4000 => { // Skip next instruction if Vx != kk.
                if self.v[(self.opcode & 0x0F00) as usize >> 8] != (self.opcode & 0x00FF) as u8 {
                    self.pc += 0x0002;
                }
                self.pc += 0x0002;
            }
            0x5000 => {
                if self.v[(self.opcode & 0x0F00) as usize >> 8] == self.v[((self.opcode & 0x00F0) as usize >> 4)]{
                    self.pc += 0x0002;
                }
                self.pc += 0x0002;
            }
            0x6000 => {
                self.v[(self.opcode & 0x0F00) as usize >> 8] = (self.opcode & 0x00FF) as u8;
                self.pc += 0x0002;
            }
            0x7000 => {
                self.v[(self.opcode & 0x0F00) as usize >> 8] += (self.opcode & 0x00FF) as u8;
                self.pc += 0x0002;
            }
            0x8000 => {
                if self.opcode & 0x000F == 0x0000 {
                    self.v[(self.opcode & 0x0F00) as usize >> 8] = self.v[((self.opcode & 0x00F0) as usize >> 4)];
                } else if self.opcode & 0x000F == 0x0001 {
                    self.v[(self.opcode & 0x0F00) as usize >> 8] |= self.v[((self.opcode & 0x00F0) as usize >> 4)];
                } else if self.opcode & 0x000F == 0x0002 {
                    self.v[(self.opcode & 0x0F00) as usize >> 8] &= self.v[((self.opcode & 0x00F0) as usize >> 4)];
                } else if self.opcode & 0x000F == 0x0003 {
                    self.v[(self.opcode & 0x0F00) as usize >> 8] ^= self.v[((self.opcode & 0x00F0) as usize >> 4)];
                } else if self.opcode & 0x000F == 0x0004 {
                    let sum = self.v[(self.opcode & 0x0F00) as usize >> 8] as u16 + self.v[(self.opcode & 0x00F0) as usize >> 4] as u16;
                    self.v[(self.opcode & 0x0F00) as usize >> 8] = (sum & 0x00FF) as u8;
                    self.v[0x000F] = if sum > 0x00FF {
                        1
                    } else {
                        0
                    };
                    
                } else if self.opcode & 0x000F == 0x0005 {
                    self.v[0x000F] = if self.v[(self.opcode & 0x0F00) as usize >> 8] > self.v[((self.opcode & 0x00F0) >> 4) as usize] {
                        1
                    } else {
                        0
                    };
                    self.v[((self.opcode & 0x0F00) >> 8) as usize] 
                        = self.v[(self.opcode & 0x0F00) as usize >> 8].wrapping_sub(self.v[(self.opcode & 0x00F0) as usize >> 4]);
                } else if self.opcode & 0x000F == 0x0006 { // SHR
                    self.v[0x000F] = self.v[(self.opcode & 0x0F00) as usize >> 8] & 0x0001;
                    self.v[(self.opcode & 0x0F00) as usize >> 8] >>= 1;
                } else if self.opcode & 0x000F == 0x0007 { // SUBN
                    self.v[0x000F] = if self.v[(self.opcode & 0x00F0) as usize >> 4] > self.v[(self.opcode & 0x0F00) as usize >> 8] {
                        1
                    } else {
                        0
                    };
                    self.v[((self.opcode & 0x0F00) >> 8) as usize] 
                        = self.v[(self.opcode & 0x00F0) as usize >> 4].wrapping_sub(self.v[(self.opcode & 0x0F00) as usize >> 8]);
                } else if self.opcode & 0x000F == 0x000E { // SHL
                    self.v[0x000F] = self.v[(self.opcode & 0x0F00) as usize >> 8] & 0x0001;
                    self.v[(self.opcode & 0x0F00) as usize >> 8] <<= 1;
                }
                self.pc += 0x0002;
            }
            0x9000 => { // SNE
                if self.v[(self.opcode & 0x0F00) as usize >> 8] != self.v[(self.opcode & 0x00F0) as usize >> 4]{
                    self.pc += 0x0002;
                }
                self.pc += 0x0002;
            }
            0xA000 => {
                self.i = self.opcode & 0x0FFF;
                self.pc += 0x0002;
            }
            0xD000 => {
                let sprite_start = self.i as usize;
                let sprite_height = (self.opcode & 0x000F) as usize;
                let x: usize = self.v[(self.opcode & 0x0F00) as usize >> 8] as usize;
                let y: usize = self.v[(self.opcode & 0x00F0) as usize >> 4] as usize;
                let display_width = WIDTH / 8;
                for i in 0..sprite_height {
                    let gfx_index = display_width * (i + y) + x;
                    println!("x,y,idx: {:04x},{:04x},{:04x}", x, y, gfx_index);
                    let old_display = self.gfx[gfx_index];
                    let new_display = self.gfx[gfx_index] ^ self.memory[sprite_start + i];
                    // set byte of the display to sprite byte
                    self.gfx[gfx_index] = new_display;
                    // set VF only if any of the pixels are unset
                    // conjunction of old value with negation of new value is non-zero if any bits are flipped
                    /*
                        0   &   !0    0
                        0   &   !1    0
                        1   &   !0    1
                        1   &   !1    0
                    */
                    self.v[0xF] = if old_display & !new_display > 0 {
                        0x001
                    } else {
                        0x000
                    };
                }
                self.pc += 0x0002;
            }
            0xF000 => {
                let reg = (self.opcode & 0x0F00) as usize >> 8;
                match self.opcode & 0x00FF {
                    0x07 => {
                        self.v[reg] = self.delay_timer;
                    }
                    0x0A => {
                        
                    }
                    0x15 => {
                        self.delay_timer = self.v[reg];
                    }
                    0x18 => {
                        self.sound_timer = self.v[reg];
                    }
                    0x1E => {
                        self.i += self.v[reg] as u16;
                    }
                    0x29 => {
                        let hex = self.v[reg];
                        match hex {
                            0 => self.i = 0x0000,
                            1 => self.i = 0x0005,
                            2 => self.i = 0x000A,
                            3 => self.i = 0x000F,
                            _ => panic!("Expecting value [0-15] at v{}, found 0x{:04x}", reg, hex)
                        }
                    }
                    0x33 => {
                        let v = self.v[reg];
                        self.memory[self.i as usize] = v / 100;
                        self.memory[self.i as usize + 1] = (v % 100) / 10;
                        self.memory[self.i as usize + 2] = v % 10;
                    }
                    0x55 => {
                        for i in 0..(self.opcode & 0x0F00 >> 8) {
                            self.memory[(self.i + i) as usize] = self.v[i as usize];
                        }
                    }
                    0x65 => {
                        for i in 0..(self.opcode & 0x0F00 >> 8) {
                            self.v[i as usize] = self.memory[(self.i + i) as usize];
                        }
                    }
                    _ => panic!("")
                }
                self.pc += 0x0002;
            }
            _ => println!("Not a valid opcode 0x{:x}", self.opcode),
        }
    }

    fn load_opcode(&mut self) {
        self.opcode = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[self.pc as usize + 1] as u16)
    }

    fn print_graphics(&self, window: &mut Window) {
        let display_width = 8;
        let mut color_buffer: Vec<u32> = Vec::with_capacity(WIDTH * HEIGHT);
        for i in 0..32 {
            let mut row = BitVec::from_bytes(&[
                self.gfx[display_width * i + 0],
                self.gfx[display_width * i + 1],
                self.gfx[display_width * i + 2],
                self.gfx[display_width * i + 3],
                self.gfx[display_width * i + 4],
                self.gfx[display_width * i + 5],
                self.gfx[display_width * i + 6],
                self.gfx[display_width * i + 7],
            ]);

            for b in row {
                if b {
                    color_buffer.push(from_u8_gray(0xFF));
                } else {
                    color_buffer.push(from_u8_gray(0x00))
                }
            }
        }

        (*window)
            .update_with_buffer(&color_buffer, WIDTH, HEIGHT)
            .unwrap();
    }

    fn load(&mut self, program: &Vec<u16>) {
        for (i, &c) in program.iter().enumerate() {
            self.memory[0x200 + 2 * i] = (c >> 8) as u8;
            self.memory[0x200 + 2 * i + 1] = (c & 0x00FF) as u8;
        }
    }

    fn load_rom(&mut self, program: &Vec<u8>) {
        for (i, &c) in program.iter().enumerate() {
            self.memory[0x200 + i] = c
        }
    }
}

fn from_u8_gray(g: u8) -> u32 {
    let g = g as u32;
    (g << 16) | (g << 8) | g
}

fn main() {
    let mut chip8 = Chip8::new();
    let program: Vec<u8> = fs::read("programs/c8_test.rom").expect("Unable to open file");

    chip8.init_mem();

    chip8.load_rom(&program);

    let mut window = match Window::new(
        "CHIP-8 - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X8,
            ..WindowOptions::default()
        },
    ) {
        Ok(win) => win,
        Err(err) => {
            println!("Unable to create window {}", err);
            return;
        }
    };

    while window.is_open() && !window.is_key_down(Key::Escape) {

        chip8.load_opcode();
        
        chip8.run();

        chip8.print_graphics(&mut window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_4xxx() {
        let mut chip8 = Chip8::new();
        chip8.init_mem();
        chip8.opcode = 0x4123;
        chip8.v[1] = 0x23;
        chip8.pc = 0x0202;

        chip8.run();

        assert_eq!(chip8.pc, 0x0206);

        // chip8.opcode = 0x4122;
        // chip8.v[1] = 0x23;
        // chip8.pc = 0x0202;

        // chip8.run();

        // assert_eq!(chip8.pc, 0x0204);

    }

    #[test]
    fn test_3xxx() {
        let mut chip8 = Chip8::new();
        chip8.init_mem();
        chip8.opcode = 0x4123;
        chip8.v[1] = 0x23;
        chip8.pc = 0x0202;

        chip8.run();

        assert_eq!(chip8.pc, 0x0204);
    }
}