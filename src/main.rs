extern crate bit_vec;
extern crate minifb;


use std::fs;
use bit_vec::BitVec;
use minifb::{Key, Scale, Window, WindowOptions};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const PROGRAM_START: usize = 0x200;

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
            pc: PROGRAM_START as u16,
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

        // 4
        interp_mem[start + 4 * sprite_ht + 0] = 0x90;
        interp_mem[start + 4 * sprite_ht + 1] = 0x90;
        interp_mem[start + 4 * sprite_ht + 2] = 0xF0;
        interp_mem[start + 4 * sprite_ht + 3] = 0x10;
        interp_mem[start + 4 * sprite_ht + 4] = 0x10;

        // 5
        interp_mem[start + 5 * sprite_ht + 0] = 0xF0;
        interp_mem[start + 5 * sprite_ht + 1] = 0x80;
        interp_mem[start + 5 * sprite_ht + 2] = 0xF0;
        interp_mem[start + 5 * sprite_ht + 3] = 0x10;
        interp_mem[start + 5 * sprite_ht + 4] = 0xF0;

        // 6
        interp_mem[start + 6 * sprite_ht + 0] = 0xF0;
        interp_mem[start + 6 * sprite_ht + 1] = 0x80;
        interp_mem[start + 6 * sprite_ht + 2] = 0xF0;
        interp_mem[start + 6 * sprite_ht + 3] = 0x90;
        interp_mem[start + 6 * sprite_ht + 4] = 0xF0;

        // 7
        interp_mem[start + 7 * sprite_ht + 0] = 0xF0;
        interp_mem[start + 7 * sprite_ht + 1] = 0x10;
        interp_mem[start + 7 * sprite_ht + 2] = 0x20;
        interp_mem[start + 7 * sprite_ht + 3] = 0x40;
        interp_mem[start + 7 * sprite_ht + 4] = 0x40;

        // 8
        interp_mem[start + 8 * sprite_ht + 0] = 0xF0;
        interp_mem[start + 8 * sprite_ht + 1] = 0x90;
        interp_mem[start + 8 * sprite_ht + 2] = 0xF0;
        interp_mem[start + 8 * sprite_ht + 3] = 0x90;
        interp_mem[start + 8 * sprite_ht + 4] = 0xF0;

        // 9
        interp_mem[start + 9 * sprite_ht + 0] = 0xF0;
        interp_mem[start + 9 * sprite_ht + 1] = 0x90;
        interp_mem[start + 9 * sprite_ht + 2] = 0xF0;
        interp_mem[start + 9 * sprite_ht + 3] = 0x10;
        interp_mem[start + 9 * sprite_ht + 4] = 0xF0;

        // 10
        interp_mem[start + 10 * sprite_ht + 0] = 0xF0;
        interp_mem[start + 10 * sprite_ht + 1] = 0x90;
        interp_mem[start + 10 * sprite_ht + 2] = 0xF0;
        interp_mem[start + 10 * sprite_ht + 3] = 0x90;
        interp_mem[start + 10 * sprite_ht + 4] = 0x90;

        // 11
        interp_mem[start + 11 * sprite_ht + 0] = 0xE0;
        interp_mem[start + 11 * sprite_ht + 1] = 0x90;
        interp_mem[start + 11 * sprite_ht + 2] = 0xE0;
        interp_mem[start + 11 * sprite_ht + 3] = 0x90;
        interp_mem[start + 11 * sprite_ht + 4] = 0xE0;

        // 12
        interp_mem[start + 12 * sprite_ht + 0] = 0xF0;
        interp_mem[start + 12 * sprite_ht + 1] = 0x80;
        interp_mem[start + 12 * sprite_ht + 2] = 0x80;
        interp_mem[start + 12 * sprite_ht + 3] = 0x80;
        interp_mem[start + 12 * sprite_ht + 4] = 0xF0;

        // 13
        interp_mem[start + 13 * sprite_ht + 0] = 0xE0;
        interp_mem[start + 13 * sprite_ht + 1] = 0x90;
        interp_mem[start + 13 * sprite_ht + 2] = 0x90;
        interp_mem[start + 13 * sprite_ht + 3] = 0x90;
        interp_mem[start + 13 * sprite_ht + 4] = 0xE0;

        // 14
        interp_mem[start + 14 * sprite_ht + 0] = 0xF0;
        interp_mem[start + 14 * sprite_ht + 1] = 0x80;
        interp_mem[start + 14 * sprite_ht + 2] = 0xF0;
        interp_mem[start + 14 * sprite_ht + 3] = 0x80;
        interp_mem[start + 14 * sprite_ht + 4] = 0xF0;

        // 15
        interp_mem[start + 15 * sprite_ht + 0] = 0xF0;
        interp_mem[start + 15 * sprite_ht + 1] = 0x80;
        interp_mem[start + 15 * sprite_ht + 2] = 0xF0;
        interp_mem[start + 15 * sprite_ht + 3] = 0x80;
        interp_mem[start + 15 * sprite_ht + 4] = 0x80;
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
                    self.pc += 0x0002;
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
                if self.v[(self.opcode & 0x0F00) as usize >> 8] == self.v[(self.opcode & 0x00F0) as usize >> 4]{
                    self.pc += 0x0002;
                }
                self.pc += 0x0002;
            }
            0x6000 => {
                self.v[(self.opcode & 0x0F00) as usize >> 8] = (self.opcode & 0x00FF) as u8;
                self.pc += 0x0002;
            }
            0x7000 => {
                let v_x = (self.opcode & 0x0F00) as usize >> 8;
                self.v[v_x] = self.v[v_x].wrapping_add((self.opcode & 0x00FF) as u8);
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
                    let gfx_index_1 = display_width * (i + y) + x / 8;
                    let gfx_index_2 = (gfx_index_1 + 1) % 8 + (display_width * (i + y));
                    // println!("x,y,idx: {:04x},{:04x},{:04x}", x, y, gfx_index);
                    let old_display_1 = self.gfx[gfx_index_1];
                    let old_display_2 = self.gfx[gfx_index_2];
                    let sprite_1 = self.memory[sprite_start + i] >> (x % 8);
                    let sprite_2 = ((self.memory[sprite_start + i] as u16) << (8 - (x % 8))) as u8;
                    let new_display_1 = self.gfx[gfx_index_1] ^ sprite_1;
                    let new_display_2 = self.gfx[gfx_index_2] ^ sprite_2;
                    // set byte of the display to sprite byte
                    self.gfx[gfx_index_1] = new_display_1;
                    self.gfx[gfx_index_2] = new_display_2;
                    // set VF only if any of the pixels are unset
                    // conjunction of old value with negation of new value is non-zero if any bits are flipped
                    /*
                        0   &   !0    0
                        0   &   !1    0
                        1   &   !0    1
                        1   &   !1    0
                    */
                    self.v[0xF] = if (old_display_1 & !new_display_1 > 0) || (old_display_2 & !new_display_2 > 0) {
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
                    0x0007 => {
                        self.v[reg] = self.delay_timer;
                    }
                    0x000A => {
                        
                    }
                    0x0015 => {
                        self.delay_timer = self.v[reg];
                    }
                    0x0018 => {
                        self.sound_timer = self.v[reg];
                    }
                    0x001E => {
                        self.i += self.v[reg] as u16;
                    }
                    0x0029 => {
                        let hex = self.v[reg];
                        match hex {
                            0x0 => self.i = 0x0000,
                            0x1 => self.i = 0x0005,
                            0x2 => self.i = 0x000A,
                            0x3 => self.i = 0x000F,
                            0x4 => self.i = 0x0014,
                            0x5 => self.i = 0x0019,
                            0x6 => self.i = 0x001E,
                            0x7 => self.i = 0x0023,
                            0x8 => self.i = 0x0028,
                            0x9 => self.i = 0x002D,
                            0xA => self.i = 0x0032,
                            0xB => self.i = 0x0037,
                            0xC => self.i = 0x003C,
                            0xD => self.i = 0x0041,
                            0xE => self.i = 0x0046,
                            0xF => self.i = 0x004B,
                            _ => panic!("Expecting value [0-15] at v{}, found 0x{:04x}", reg, hex)
                        }
                    }
                    0x0033 => {
                        let v = self.v[reg];
                        self.memory[self.i as usize] = v / 100;
                        self.memory[self.i as usize + 1] = (v % 100) / 10;
                        self.memory[self.i as usize + 2] = v % 10;
                    }
                    0x0055 => {
                        for i in 0..(self.opcode & 0x0F00 >> 8) {
                            self.memory[(self.i + i) as usize] = self.v[i as usize];
                        }
                    }
                    0x0065 => {
                        let x = self.opcode & 0x0F00 >> 8;
                        for i in 0..x {
                            self.v[i as usize] = self.memory[(self.i + i) as usize];
                        }
                        self.i += x + 1;
                    }
                    _ => panic!("Opcode not implemented: {:04x}", self.opcode)
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
            let row = BitVec::from_bytes(&[
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

    fn load_rom(&mut self, program: &Vec<u8>) {
        for (i, &c) in program.iter().enumerate() {
            self.memory[PROGRAM_START + i] = c
        }
    }
}

fn from_u8_gray(g: u8) -> u32 {
    let g = g as u32;
    (g << 16) | (g << 8) | g
}

fn main() {
    let mut chip8 = Chip8::new();
    let program: Vec<u8> = fs::read("programs/cycle-numbers.rom").expect("Unable to open file");

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

        assert_eq!(chip8.pc, 0x0204);

        chip8.opcode = 0x4123;
        chip8.v[1] = 0x22;
        chip8.pc = 0x0202;

        chip8.run();

        assert_eq!(chip8.pc, 0x0206);

    }

    #[test]
    fn test_3xxx() {
        let mut chip8 = Chip8::new();
        chip8.init_mem();
        chip8.opcode = 0x3123;
        chip8.v[1] = 0x23;
        chip8.pc = 0x0202;

        chip8.run();

        assert_eq!(chip8.pc, 0x0206);

        chip8.opcode = 0x3123;
        chip8.v[1] = 0x22;
        chip8.pc = 0x0202;

        chip8.run();

        assert_eq!(chip8.pc, 0x0204);
    }

    #[test]
    fn test_5xxx() {
        let mut chip8 = Chip8::new();
        chip8.init_mem();
        chip8.opcode = 0x5123;
        chip8.v[1] = 0x23;
        chip8.v[2] = 0x23;
        chip8.pc = 0x0202;

        chip8.run();

        assert_eq!(chip8.pc, 0x0206);
        chip8.opcode = 0x5123;
        chip8.v[1] = 0x23;
        chip8.v[2] = 0x22;
        chip8.pc = 0x0202;

        chip8.run();

        assert_eq!(chip8.pc, 0x0204);
    }
}