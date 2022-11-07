use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

#[repr(u8)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    BLACK,
    BLUE,
    GREEN,
    CYAN,
    RED,
    MAGENTA,
    BROWN,
    LIGHTGRAY,
    DARKGRAY,
    LIGHTBLUE,
    LIGHTGREEN,
    LIGHTCYAN,
    LIGHTRED,
    PINK,
    YELLOW,
    WHITE,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ColorMode(u8);

impl ColorMode {
    fn new(foreground: Color, background: Color) -> ColorMode {
        // NOTE:
        //  - background * (2 ** 4)
        //  - bitwise OR
        //      0110
        //      0001
        //      ----
        //      0111
        ColorMode((background as u8) << 4 | (foreground as u8))
    }
}

// NOTE: repr(C) ensures struct field order
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScreenChar {
    ascii_character: u8,
    color_mode: ColorMode,
}

// NOTE: default matrix size of the I/O VGA buffer
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    // NOTE: volatile is necessary to prevent compiler optimizations
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    col_position: usize,
    color_mode: ColorMode,
    buffer: &'static mut Buffer,
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);

        Ok(())
    }
}

impl Writer {
    fn clear_row(&mut self, row: usize) {
        let blank_char = ScreenChar {
            ascii_character: b' ',
            color_mode: self.color_mode,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank_char);
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();

                self.buffer.chars[row - 1][col].write(character);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.col_position = 0;
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                let row = BUFFER_HEIGHT - 1;
                let col = self.col_position;
                let color_mode = self.color_mode;

                if self.col_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_mode,
                });

                self.col_position += 1;
            }
        }
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0x3f), // NOTE: 0x3f == question mark
            }
        }
    }
}

// NOTE: lazy_static call is to make a non-const function as const on compile time
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        col_position: 0,
        color_mode: ColorMode::new(Color::YELLOW, Color::BLACK),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    WRITER.lock().write_fmt(args).unwrap();
}

#[test_case]
fn test_println_single() {
    println!("test_println_single output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    let s = "some test string that fits on a single line";

    println!("{}", s);

    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();

        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}
