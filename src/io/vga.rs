use core::fmt::Write;

extern crate volatile;
use self::volatile::Volatile;

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 20;

#[repr(transparent)]
struct Buffer {
    addr: [[Volatile<ScreenChar>; VGA_WIDTH]; VGA_HEIGHT],
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)] #[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl VgaDisplay {
    fn backspace(&mut self) -> () {
        if self.row == 0 && self.col == 0 {
            return;
        }
        if self.col == 0 {
            self.row-=1;
            self.col=VGA_WIDTH;
        }
        self.col-=1;
        self.putchar(self.col, self.row, ' ', ColorCode::new(Color::White, Color::Black)); // TODO Support colors other than white
        self.col-=1;
    }
    fn putchar(&mut self, x: usize, y: usize, c: char, color: ColorCode) -> () {
        self.buffer.addr[y][x].write(ScreenChar {
            ascii_character: c as u8,
            color_code: color
        });
    }
}

impl Default for VgaDisplay {
    fn default() -> VgaDisplay {
        let mut _row = 0;
        let mut _col = 0;

        let mut res = VgaDisplay {
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
            row: _row,
            col: _col
        };

        // Clears the screen
        for y in 0..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                res.putchar(x, y, ' ', ColorCode::new(Color::White, Color::Black));
            }
        }
        return res;
    }

}

impl Write for VgaDisplay {

    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() { 
            self.write_char(c).unwrap();
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        match c {
            '\x0A' => {
                self.row+=1;
                self.col=0;
                Ok(())
            },
            '\x08' => { // TODO is this backspace?
                self.backspace();
                Ok(())
            },
            c => {
                self.putchar(self.col, self.row, c, ColorCode::new(Color::White, Color::Black)); // TODO Support colors other than white
                self.col+=1;
                if self.col >= VGA_WIDTH {
                    self.col = 0;
                    self.row += 1;
                }
                Ok(())
            }
        }
    }
}
