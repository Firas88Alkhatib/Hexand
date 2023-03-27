use bootloader_api::info::{ FrameBufferInfo, PixelFormat };
use conquer_once::spin::OnceCell;
use core::{ fmt, ptr };
use spinning_top::{ lock_api::Mutex, RawSpinlock, Spinlock };

// supoort only psf1 currently
// refer to https://en.wikipedia.org/wiki/PC_Screen_Font
const FONT: &'static [u8] = include_bytes!("fonts/Uni2-Fixed16.psf");
const HEADER_SIZE: usize = FONT[1] as usize;
const FONT_HEADER: &[u8] = &FONT[0..HEADER_SIZE];
const CHAR_SIZE: usize = FONT_HEADER[3] as usize;
const CHARS_DATA: &[u8] = &FONT[HEADER_SIZE..512 * CHAR_SIZE];
const UNICODE_TABLE: &[u8] = &FONT[HEADER_SIZE + CHARS_DATA.len()..];
const CHAR_HEIGHT: usize = CHAR_SIZE;
const CHAR_WIDTH: usize = 8 as usize; // in psf 1 the width is always 8

const LINE_SPACING: usize = 3;
const LETTER_SPACING: usize = 1;
const SCREEN_PADDING: usize = 5;
const BACKUP_CHAR: char = '?';

// const IMG: &'static [u8] = include_bytes!("./images/forest.bmp");

pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}
// r: 203, g: 58, b: 55, a: 0   // red
// rgb(43, 116, 201)
// r: 73, g: 136, b: 221, a: 0 // bright blue nice!
const COLOR: Color = Color { r: 243, g: 98, b: 95, a: 0};

pub struct FrameBufferWriter {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
}

impl FrameBufferWriter {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut frame_buffer_writer = Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
        };
        frame_buffer_writer.clear();
        frame_buffer_writer
    }
    fn width(&self) -> usize {
        self.info.width
    }

    fn height(&self) -> usize {
        self.info.height
    }
    fn newline(&mut self) {
        self.y_pos += CHAR_HEIGHT + LINE_SPACING;
        self.carriage_return()
    }

    fn carriage_return(&mut self) {
        self.x_pos = SCREEN_PADDING;
    }
    pub fn clear(&mut self) {
        self.x_pos = SCREEN_PADDING;
        self.y_pos = SCREEN_PADDING;
        self.framebuffer.fill(0);
    }
    fn get_char_position(&mut self, char: char) -> Option<usize> {
        let mut code_index = 0;
        // unicode table is groups of u16 items, every group ends when the value of the item is 0xFFFF.
        // the first group points to the first glyph data , the send points to the second glyph and so on.
        // a group has items that all point to the same glyph, for example:
        // 0x0041 -- U+0041  'A'
        // 0x00C4 -- U+00C4  'Ä'
        // 0x00C5 -- U+00C5  'À'
        // 0xFFFF
        // all those unicode chars point to the same glyph in the font data
        // which could be for example the char 'A' depending on the font.
        for i in 0..UNICODE_TABLE.len() / 2 {
            let value = u16::from_le_bytes([UNICODE_TABLE[i * 2], UNICODE_TABLE[i * 2 + 1]]);
            if value == 0xffff {
                code_index += 1;
            } else if value == (char as u16) {
                return Some(code_index * CHAR_SIZE);
            }
        }
        return None;
    }
    fn get_glyph_data(&mut self, char_code: char) -> Option<&'static [u8]> {
        let char_start = self.get_char_position(char_code)?;
        let char_end = char_start + CHAR_SIZE;
        if char_end <= CHARS_DATA.len() {
            Some(&CHARS_DATA[char_start..char_end])
        } else {
            None
        }
    }
    pub fn write_pixel(&mut self, x: usize, y: usize, color: Color) {
        let pixel_offset = y * self.info.stride + x;

        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [color.r, color.g, color.b, color.a],
            PixelFormat::Bgr => [color.b, color.g, color.r, color.a],
            other => {
                // set a supported (but invalid) pixel format before panicking to avoid a double
                // panic; it might not be readable though
                self.info.pixel_format = PixelFormat::Rgb;
                panic!("pixel format {:?} not supported in logger", other)
            }
        };
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer[byte_offset..byte_offset + bytes_per_pixel].copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.framebuffer[byte_offset]) };
    }
    fn render_char(&mut self, char: char) {
        let glyph = self.get_glyph_data(char).unwrap_or_else(|| self.get_glyph_data(BACKUP_CHAR).unwrap());
        for row in 0..CHAR_HEIGHT {
            for col in 0..CHAR_WIDTH {
                let index = row * CHAR_WIDTH + col;
                let bit = glyph[index / 8] & (1 << (7 - (index % 8)));
                if bit != 0 {
                    self.write_pixel(self.x_pos + col, self.y_pos + row, COLOR);
                }
            }
        }

        self.x_pos += 8 + LETTER_SPACING;
    }
    fn write_char(&mut self, char: char) {
        match char {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            char => {
                let new_xpos = self.x_pos + CHAR_WIDTH;
                if new_xpos >= self.width() {
                    self.newline();
                }
                let new_ypos = self.y_pos + CHAR_HEIGHT + SCREEN_PADDING;
                if new_ypos >= self.height() {
                    self.clear();
                }
                self.render_char(char);
            }
        }
    }
    // fn image(&mut self) {
    //     let bpm_pixeldata_offset: usize = 54;
    //     let bytes_per_pixel: usize = 3;
    //     let pixel_data = &IMG[bpm_pixeldata_offset..];
    //     let width = 1000;
    //     let height = 667;

    //     for y in 0..height {
    //         for x in 0..width {
    //             let index = ((height - y - 1) * width + x) as usize * bytes_per_pixel;
    //             let b = pixel_data[index];
    //             let g = pixel_data[index + 1];
    //             let r = pixel_data[index + 2];
    //             self.write_pixel(x as usize, y as usize, Color { r, g, b, a: 0 });
    //         }
    //     }
    // }
}

unsafe impl Send for FrameBufferWriter {}
unsafe impl Sync for FrameBufferWriter {}

impl fmt::Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}
pub static WRITER: OnceCell<Spinlock<FrameBufferWriter>> = OnceCell::uninit();

pub fn init(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> &Mutex<RawSpinlock, FrameBufferWriter> {
    WRITER.get_or_init(move || Spinlock::new(FrameBufferWriter::new(framebuffer, info)))
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::frame_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.try_get().unwrap().lock().write_fmt(args).unwrap();
}

// pub fn image() {
//     WRITER.try_get().unwrap().lock().image();
// }