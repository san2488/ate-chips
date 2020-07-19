extern crate bit_vec;
extern crate minifb;

use std::process;
use std::fs::File;
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
    stack: [u8; 6],
    sp: u8,
    opcode: u16,
    gfx: [u8; 8 * 32],
    pc: usize,
    i: u16,
}

impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {
            delay_timer: 0,
            sound_timer: 0,
            memory: [0; 4096],
            opcode: 0,
            sp: 0,
            gfx: [0x000; WIDTH / 8 * HEIGHT],
            v: [0; 16],
            stack: [0; 6],
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
                }
            }
            0x6000 => {
                self.v[((self.opcode & 0x0F00) >> 8) as usize] = (self.opcode & 0x00FF) as u8;
                self.pc += 0x0002;
            }
            0xA000 => {
                self.i = self.opcode & 0x0FFF;
                self.pc += 0x0002;
            }
            0x7000 => {
                self.v[((self.opcode & 0x0F00) >> 8) as usize] += (self.opcode & 0x00FF) as u8;
                self.pc += 0x0002;
            }
            0xD000 => {
                let sprite_start = self.i as usize;
                let sprite_height = (self.opcode & 0x000F) as usize;
                let x: usize = self.v[((self.opcode & 0x0F00) >> 8) as usize] as usize;
                let y: usize = self.v[((self.opcode & 0x00F0) >> 4) as usize] as usize;
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
                let reg = ((self.opcode & 0x0F00) >> 8) as usize;
                let hex = self.v[reg];
                println!("hex: 0x{:04x}", hex);
                match hex {
                    0 => self.i = 0x0000,
                    1 => self.i = 0x0005,
                    2 => self.i = 0x000A,
                    3 => self.i = 0x000F,
                    _ => panic!("Expecting value [0-15] at v{}, found 0x{:04x}", reg, hex)
                }
                self.pc += 2;
            }
            0x0000 => {
                process::exit(0x0100);
            }
            _ => println!("Not a valid opcode 0x{:x}", self.opcode),
        }
    }

    fn load_opcode(&mut self) {
        self.opcode = (self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1] as u16)
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
}

fn from_u8_gray(g: u8) -> u32 {
    let g = g as u32;
    (g << 16) | (g << 8) | g
}

fn main() {
    let mut chip8 = Chip8::new();
    let file = File::open("programs/mictest-123.txt").expect("Unable to open file");
    let reader = BufReader::new(file);
    let program: Vec<u16> = reader.lines()
        .map(|l| l.expect("Could not parse line"))
        .flat_map(|l| l.split_whitespace().skip(1).map(String::from).collect::<Vec<String>>())
        .map(|c| u16::from_str_radix(&c, 16).unwrap())
        .collect();
    println!("{:04x?}", program);

    chip8.init_mem();

    chip8.load(&program);

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
