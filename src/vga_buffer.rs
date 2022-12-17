//! This module is an interface using the VGA text mode 
//! to print text to to screen.

/// all allowed colour constants, 
/// stored as an `8-bit` unsigned integer
/// 
/// colours are actually only `4-bits`,
/// so there are 4 trailing zeros
#[allow(dead_code)]
#[allow(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Colour {
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

/// a colour code consisting of 2 `4-bit` colours:
/// `foreground|background`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColourCode(u8);

impl ColourCode {
    /// creates a new `8-bit` colour code
    fn new(foreground: Colour, background: Colour) -> ColourCode {
        ColourCode((background as u8) << 4 | (foreground as u8))
    }
}


/// struct containing information for how characters 
/// will be displayed on the screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    colour_code: ColourCode,
}


use volatile::Volatile;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// screen buffer - which is a 2D array of 
/// `rows` and `columns` of characters 
/// to be displayed on the screen
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}


/// controls writing to the buffer
pub struct Writer {
    column_position: usize,
    colour_code: ColourCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// write an individual byte to the screen at the current
    /// position (most recent row + current column)
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => self.write_char_byte(byte),
        }
    }

    /// write a byte to the buffer when it's not `\n`
    fn write_char_byte(&mut self, byte: u8) {
        // current line is full
        if self.column_position >= BUFFER_WIDTH {
            self.new_line();
        }

        let row = BUFFER_HEIGHT-1;
        let col = self.column_position;

        let colour_code = self.colour_code;
        self.buffer.chars[row][col].write(ScreenChar {
            ascii_character: byte,
            colour_code,
        });
        self.column_position += 1;
    }

    /// moves the writer to the next line, 
    /// shifting all lines up by 1
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// makes the given row blank
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            colour_code: self.colour_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    /// writes an entire string slice to the buffer
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }
}

use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        colour_code: ColourCode::new(Colour::Pink, Colour::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }
    });
}


/// print to the buffer
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// print to the buffer, writing a new line after the given input
#[macro_export]
macro_rules! println {
    () => (crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// writes the input to the writer
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}