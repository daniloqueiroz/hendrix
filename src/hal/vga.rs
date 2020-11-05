use core::fmt;

use crate::kernel::console::ConsolePrinter;
use core::fmt::{Arguments, Write};
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::interrupts;

const TEXT_BUFFER_LINES: usize = 25;
const TEXT_BUFFER_COLS: usize = 80;

// The standard color palette in VGA text mode.
#[allow(dead_code)]
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

/// A ColorSchema is a combination of a foreground and a background color.
/// The first 4 bits correspond to the background color
/// and the last 4 bits to the foreground color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorScheme(u8);

impl ColorScheme {
    // Create a new `ColorScheme` with the given foreground and background colors.
    fn new(foreground: Color, background: Color) -> ColorScheme {
        ColorScheme((background as u8) << 4 | (foreground as u8))
    }
}

/// A screen character in the VGA text buffer, consisting of an ASCII character and a `ColorCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorScheme,
}

/// A structure representing the VGA text buffer.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; TEXT_BUFFER_COLS]; TEXT_BUFFER_LINES],
}

/// A writer type that allows writing ASCII bytes and strings to an underlying `Buffer`.
///
/// Wraps lines at `BUFFER_WIDTH`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct Writer {
    column_position: usize,
    color_code: ColorScheme,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Writes an ASCII byte to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= TEXT_BUFFER_COLS {
                    self.new_line();
                }

                let row = TEXT_BUFFER_LINES - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character. Does **not**
    /// support strings with non-ASCII characters, since they can't be printed in the VGA text
    /// mode.
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Shifts all lines one line up and clears the last row.
    fn new_line(&mut self) {
        for row in 1..TEXT_BUFFER_LINES {
            for col in 0..TEXT_BUFFER_COLS {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(TEXT_BUFFER_LINES - 1);
        self.column_position = 0;
    }

    /// Clears a row by overwriting it with blank characters.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..TEXT_BUFFER_COLS {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub struct VGA {
    writer: Mutex<Writer>,
}

impl VGA {
    pub fn new() -> Self {
        return Self {
            writer: Mutex::new(Writer {
                column_position: 0,
                color_code: ColorScheme::new(Color::LightGreen, Color::Black),
                buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
            }),
        };
    }
}

impl ConsolePrinter for VGA {
    fn print(&self, args: Arguments) {
        interrupts::without_interrupts(|| {
            self.writer.lock().write_fmt(args).unwrap();
        });
    }
}

// Unit tests for the VGA/Writer
#[cfg(test)]
mod tests {
    use crate::hal::vga::{TEXT_BUFFER_LINES, VGA};
    use crate::kernel::console::ConsolePrinter;
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    #[test_case]
    fn test_println_simple() {
        let vga = VGA::new();
        vga.print(format_args!("test_println_simple output"));
    }

    #[test_case]
    fn test_println_many() {
        let vga = VGA::new();
        for _ in 0..200 {
            vga.print(format_args!("test_println_many output"));
        }
    }

    #[test_case]
    fn test_println_output() {
        let vga = VGA::new();
        let s = "Some test string that fits on a single line";

        interrupts::without_interrupts(|| {
            let mut writer = vga.writer.lock();
            writeln!(writer, "\n{}", s).expect("writeln failed");
            for (i, c) in s.chars().enumerate() {
                let screen_char = writer.buffer.chars[TEXT_BUFFER_LINES - 2][i].read();
                assert_eq!(char::from(screen_char.ascii_character), c);
            }
        });
    }
}
