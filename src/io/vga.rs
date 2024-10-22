/*
 * Simple VGA Buffer implementation using the BIOS VGA Buffer
 */

use core::fmt::Write;
use spin::{Lazy, Mutex};
use volatile::Volatile;

use crate::status::ErrorCode;

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
pub static SCREEN: Lazy<Mutex<VgaDisplay>> =
    Lazy::new(|| Mutex::new(VgaDisplay::new().expect("Failed to initialize VGA")));

#[repr(transparent)]
struct Buffer {
    addr: [[Volatile<ScreenChar>; VGA_WIDTH]; VGA_HEIGHT],
}

pub struct VgaDisplay {
    buffer: &'static mut Buffer,
    row: usize,
    col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl VgaDisplay {
    fn backspace(&mut self) -> Result<(), ErrorCode> {
        if self.row == 0 && self.col == 0 {
            return Ok(());
        }
        if self.col == 0 {
            self.row -= 1;
            self.col = VGA_WIDTH;
        }
        self.col -= 1;

        // TODO support multi color
        self.putchar(
            self.col,
            self.row,
            ' ',
            ColorCode::new(Color::White, Color::Black),
        )?;

        self.col -= 1;
        Ok(())
    }
    fn putchar(&mut self, x: usize, y: usize, c: char, color: ColorCode) -> Result<(), ErrorCode> {
        self.buffer
            .addr
            .get_mut(y)
            .ok_or(ErrorCode::OutOfBounds)?
            .get_mut(x)
            .ok_or(ErrorCode::OutOfBounds)?
            .write(ScreenChar {
                ascii_character: c as u8,
                color_code: color,
            });
        Ok(())
    }
    fn clear(&mut self) -> Result<(), ErrorCode> {
        for y in 0..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                self.putchar(x, y, ' ', ColorCode::new(Color::White, Color::Black))?;
            }
        }
        self.row = 0;
        self.col = 0;
        Ok(())
    }

    pub fn new() -> Result<Self, ErrorCode> {
        let mut display = Self {
            // TODO this will not work with monochrome monitors since their address is 0xB0000
            // See https://wiki.osdev.org/Detecting_Colour_and_Monochrome_Monitors
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
            row: 0,
            col: 0,
        };
        display.clear()?;

        Ok(display)
    }
}

impl Write for VgaDisplay {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        match c {
            '\x0A' => {
                self.row += 1;
                // TODO support scroll buffer
                self.col = 0;
                Ok(())
            }
            '\x08' => {
                // TODO is this backspace?
                self.backspace().map_err(|_| core::fmt::Error)?;
                Ok(())
            }
            c => {
                // TODO support multi color
                self.putchar(
                    self.col,
                    self.row,
                    c,
                    ColorCode::new(Color::White, Color::Black),
                )
                .map_err(|_| core::fmt::Error)?;

                self.col += 1;
                if self.col >= VGA_WIDTH {
                    self.col = 0;
                    self.row += 1;
                }
                Ok(())
            }
        }
    }
}
