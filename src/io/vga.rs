use core::fmt::Write;
//extern crate alloc;
//use alloc::boxed::Box;

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 20;

#[repr(transparent)]
struct Buffer {
    addr: &'static mut [[ScreenChar; VGA_WIDTH]; VGA_HEIGHT]
}

pub struct VgaDisplay {
    buffer: &'static mut Buffer,
    row: usize,
    col: usize
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
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl VgaDisplay {
    pub fn new() -> VgaDisplay {
        let mut _row = 0;
        let mut _col = 0;

        VgaDisplay {
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
            row: _row,
            col: _col
        }
    }
    fn backspace(&mut self) -> core::fmt::Result {
        if self.row == 0 && self.col == 0 {
            return Ok(());
        }
        if self.col == 0 {
            self.row-=1;
            self.col=VGA_WIDTH;
        }
        self.col-=1;
        self.write_char(' ');
        self.col-=1;
        return Ok(()); 
    }
    fn putchar(&mut self, x: usize, y: usize, c: char, color: ColorCode) -> () {
        self.buffer.addr[y][x] = ScreenChar {
            ascii_character: c as u8,
            color_code: color
        };
    }
}

impl Write for VgaDisplay {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unimplemented!();
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        match c {
            '\x0A' => { // TODO is this \n?
                self.row+=1;
                self.col=0;
                Ok(())
            },
            '\x08' => { // TODO is this backspace?
                self.backspace();
                Ok(())
            },
            c => {
                self.putchar(self.col, self.row, c, ColorCode::new(Color::Black, Color::White)); // TODO Support colors other than white
                self.col+=1;
                if self.col >= VGA_WIDTH {
                    self.col = 0;
                    self.row += 1;
                }
                Ok(())
            }
        }
    }

    fn write_fmt(mut self: &mut Self, args: core::fmt::Arguments<'_>) -> core::fmt::Result {
        unimplemented!();
    }
}
