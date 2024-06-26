// This file probably shouldn't be called "screen" since it contains a lot more than
// just the screen abstraction. It also contains the rendering logic.

use std::sync::mpsc::{channel, Sender};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;

#[cfg(target_family = "unix")]
use super::system::{termios as term, unistd};
use std::ops::{Index, IndexMut};

// A basic representation of a "pixel"
#[derive(Clone, PartialEq, Debug)]
pub struct Pixel {
    // The shape of the pixel, as in what is printed out when the pixel is displayed.
    // It is a two-element array because every "pixel" takes up two characters on the
    // terminal.
    pub shape: [char; 2],

    // The color of the pixel
    pub color: Color,
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            shape: [' ', ' '],
            color: Color::Default,
        }
    }
}

// Support for RGB
#[derive(Clone, PartialEq, Debug)]
pub enum Color {
    Default,
    Basic(u8), // Basic color support. Use for maximum compatibility. Only have 16 colors available.
}

pub mod colors {
    pub mod basic {
        pub const _BLACK: u8 = 30;
        pub const RED: u8 = 31;
        pub const GREEN: u8 = 32;
        pub const YELLOW: u8 = 33;
        pub const BLUE: u8 = 34;
        pub const MAGENTA: u8 = 35;
        pub const CYAN: u8 = 36;
        pub const _WHITE: u8 = 37;

        // Bright colors.
        pub const BRIGHT_BLACK: u8 = 90;
        pub const _BRIGHT_RED: u8 = 91;
        pub const _BRIGHT_GREEN: u8 = 92;
        pub const BRIGHT_YELLOW: u8 = 93;
        pub const _BRIGHT_BLUE: u8 = 94;
        pub const _BRIGHT_MAGENTA: u8 = 95;
        pub const _BRIGHT_CYAN: u8 = 96;
        pub const _BRIGHT_WHITE: u8 = 97;
    }
}

// A basic abstraction of a screen that makes it easier to render bitmap graphics
// on the terminal
pub struct Screen {
    width: u32,
    height: u32,

    has_cursor_moved: bool,

    event_reciever: Receiver<char>,

    // Used a single-dimensional vector instead of a vector of vectors to improve
    // performance.
    pixels: Vec<Pixel>,
}
//
// Basically, read whatever key the user has pressed from the terminal
// This is the UNIX version. The Windows version uses Microsoft's dedicated
// function instead of getchar.
#[cfg(target_family = "unix")]
fn read_input() -> Option<char> {
    use std::io::Read;

    let mut character = 0;
    match std::io::stdin().read(std::slice::from_mut(&mut character)) {
        Ok(bytes_read) => {
            if bytes_read != 0 {
                std::char::from_u32(character.into())
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

// The Windows version of read input. Basically does the exact same
// thing, but for windows.
#[cfg(target_family = "windows")]
fn read_input() -> Option<char> {
    unsafe { char::from_u32(crate::system::conio::_getch().try_into().ok()?) }
}

// This is the thread that constantly listens for keyboard events and
// broadcasts them as soon as it hears one.
fn event_thread(sender: Sender<char>) {
    loop {
        if let Some(character) = read_input() {
            if let Err(error) = sender.send(character) {
                eprintln!("\x1B[91m[ERROR]: {:?}\x1B[91m", error);
            }
        }
    }
}

impl Screen {
    // Creates a blank screen with a certain width and height.
    pub fn new(width: u32, height: u32) -> Result<Screen, std::num::TryFromIntError> {
        // Not very clean, but this is very hacky so don't get mad.
        #[cfg(target_family = "unix")]
        unsafe {
            let mut terminal_settings = term::termios::default();
            term::tcgetattr(unistd::STDIN_FILENO as i32, &mut terminal_settings);

            terminal_settings.c_lflag &= !term::ICANON;
            terminal_settings.c_lflag &= !term::ECHO;

            term::tcsetattr(
                unistd::STDIN_FILENO as i32,
                term::TCSANOW as i32,
                &terminal_settings,
            );
        }

        let (sender, event_reciever) = channel();

        // Make sure to start the event thread after creating the screen.
        thread::spawn(move || event_thread(sender));

        // And, yes, the thread runs until the program itself stops.
        // That's probably not a good idea but it's the best we've got.

        Ok(Screen {
            width,
            height,
            event_reciever,
            has_cursor_moved: false,
            pixels: vec![Pixel::default(); (width * height).try_into()?],
        })
    }

    // Returns the width of the screen. This can be used by clients to ensure
    // that they don't try to write to pixels that are out of bounds, which
    // can cause the program to panic.
    pub fn _width(&self) -> u32 {
        self.width
    }

    // Returns the height of the screen. Same use case as the width() function
    pub fn _height(&self) -> u32 {
        self.height
    }

    // Clears the entire screen. Basically, reset everyting back to spaces.
    pub fn clear(&mut self) {
        // This should be self-explanatory, but if you're the kind of person who
        // is too lazy to read any code, what this basically does is it iterates
        // over all the pixels and resets them.
        for pixel in self.pixels.iter_mut() {
            *pixel = Pixel::default()
        }
    }

    // This function would likely never be used in this program but I added it here
    // anyways for completeness's sake. And with the hope that, in the future, I might
    // be able to copy and paste this file for another project.
    //
    // By the way, after resizing the screen would be blank so...
    pub fn _resize(&mut self, new_width: u32, new_height: u32) {
        self.width = new_width;
        self.height = new_height;
        self.pixels.resize(
            (new_width * new_height).try_into().unwrap(),
            Pixel::default(),
        );
    }

    // Takes the first event from the event channel and return it if it exists. If there
    // is no event, it will return an Err variant.
    pub fn read_input(&self) -> Result<char, TryRecvError> {
        self.event_reciever.try_recv()
    }

    // Finally, the function that you've all been waiting for. This guy does all of the
    // hard work of going through the pixels and drawing them on the terminal.
    pub fn present(&mut self) {
        if self.has_cursor_moved {
            println!("\x1B[{}D\x1B[{}A", self.width, self.height + 1);
        }

        // Move to the start of the screen before printing.
        // print!("\x1B[H");

        for i in 0..self.height {
            for j in 0..self.width {
                let pixel: &Pixel = &self[j][i as usize];

                match pixel.color {
                    Color::Basic(code) => {
                        print!("\x1B[{}m{}{}\x1B[0m", code, pixel.shape[0], pixel.shape[1])
                    }
                    Color::Default => print!("{}{}", pixel.shape[0], pixel.shape[1]),
                }
            }

            println!("");
        }

        self.has_cursor_moved = true;
    }
}

// In Tetris, all shapes are made up of only 4 pixels.
const SHAPE_PIXEL_COUNT: usize = 4;

// A struct for a shape.
#[derive(Clone, PartialEq, Debug)]
pub struct Shape {
    // The squares that are taken up by the shape, relative to the
    // shape itself.
    pub pixels: [(i16, i16); SHAPE_PIXEL_COUNT],
    // The pixel to fill the shape with.
    pub fill_pixel: Pixel,
}

impl Shape {
    pub fn rotate(&mut self, rotate_left: bool) {
        self.pixels.iter_mut().for_each(|coordinates| {
            if rotate_left {
                let (old_x, old_y) = *coordinates;
                let (new_x, new_y) = coordinates;

                *new_x = -old_y;
                *new_y = old_x;
            } else {
                let (old_x, old_y) = *coordinates;
                let (new_x, new_y) = coordinates;

                *new_x = old_y;
                *new_y = -old_x;
            }
        })
    }

    pub fn flip(&mut self, horizontally: bool) {
        self.pixels.iter_mut().for_each(|coordinates| {
            let (old_x, old_y) = *coordinates;
            let (new_x, new_y) = coordinates;

            if horizontally {
                *new_x = -old_x;
            } else {
                *new_y = -old_y;
            }
        })
    }

    pub fn is_within_bounds(&self, x: u16, y: u16) -> (bool, bool) {
        use crate::tetris::{GAME_HEIGHT, GAME_WIDTH};

        let mut within_x_bounds = true;
        let mut within_y_bounds = true;

        self.pixels.iter().for_each(|(block_x, block_y)| {
            let block_x: i16 = block_x + <u16 as TryInto<i16>>::try_into(x).unwrap();
            let block_y: i16 = block_y + <u16 as TryInto<i16>>::try_into(y).unwrap();

            if block_y >= GAME_HEIGHT as i16 || block_y <= 0 {
                within_y_bounds = false;
            }

            if block_x > GAME_WIDTH as i16 || block_x <= 0 {
                within_x_bounds = false;
            }
        });

        (within_x_bounds, within_y_bounds)
    }
}

// TODO: Actually implement some methods to make this useful.
#[derive(Debug)]
pub struct OutOfBoundsError {}

// Alright, so for the purpose of organization, I'm going to put
// the high-level rendering logics into a different implementation
// block.
impl Screen {
    // Draws a shape.
    pub fn draw_shape(&mut self, shape: &Shape, x_pos: u16, y_pos: u16, ghost: bool) {
        let x_pos: i16 = x_pos.try_into().unwrap();
        let y_pos: i16 = y_pos.try_into().unwrap();

        shape.pixels.iter().for_each(|(pixel_x, pixel_y)| {
            let real_x: i16 = x_pos + *pixel_x;
            let real_y: i16 = y_pos + *pixel_y;

            // Any pixels that are out of bounds are automatically clipped off.
            // Also, the casting is safe as the || operators are short-circuited.
            if (real_x < 0 || real_x as u32 >= self.width)
                || (real_y < 0 || real_y as u32 >= self.height)
            {
                return;
            }

            // Both u32 and usize are larger than i16 and real_x and real_y are both
            // guaranteed to be positive by this point, so this is safe (at least it
            // should be).
            self[real_x as u32][real_y as usize] = shape.fill_pixel.clone();

            if ghost {
                self[real_x as u32][real_y as usize].shape =
                    [crate::unicode::LIGHT_SHADE, crate::unicode::LIGHT_SHADE];
            }
        })
    }

    // Fills the screen with a specific color.
    pub fn _fill_with_pixel(&mut self, pixel: &Pixel) {
        for i in 0..self.width {
            for j in 0..self.height {
                self[i][j as usize] = pixel.clone();
            }
        }
    }

    // Fills an area of the screen with a specific color.
    pub fn fill_area_with_pixel(
        &mut self,
        pixel: &Pixel,
        start_x: u16,
        start_y: u16,
        end_x: u16,
        end_y: u16,
    ) {
        for i in start_x..end_x {
            for j in start_y..end_y {
                self[i.into()][j as usize] = pixel.clone();
            }
        }
    }

    // Draws a box. Duh.
    pub fn draw_box(
        &mut self,
        x_pos: u16,
        y_pos: u16,
        width: u16,
        height: u16,
    ) -> Result<(), OutOfBoundsError> {
        let left = x_pos;
        let right = x_pos + width;

        let top = y_pos;
        let bottom = y_pos + height;

        if <u16 as Into<u32>>::into(right) >= self.width
            || <u16 as Into<u32>>::into(bottom) >= self.height
        {
            return Err(OutOfBoundsError {});
        }

        // There are probably better ways to do this with iterators but I don't
        // know them so please let me know if you do know them. :ye:

        // Draw the horizontal lines
        for i in left..right {
            use crate::unicode::BOX_DRAWINGS_LIGHT_HORIZONTAL;

            self[i.into()][<u16 as Into<usize>>::into(top)] = Pixel {
                shape: [BOX_DRAWINGS_LIGHT_HORIZONTAL, BOX_DRAWINGS_LIGHT_HORIZONTAL],
                color: Color::Default,
            };

            self[i.into()][<u16 as Into<usize>>::into(bottom)] = Pixel {
                shape: [BOX_DRAWINGS_LIGHT_HORIZONTAL, BOX_DRAWINGS_LIGHT_HORIZONTAL],
                color: Color::Default,
            };
        }

        // Draw the vertical lines.
        for i in top..bottom {
            use crate::unicode::BOX_DRAWINGS_LIGHT_VERTICAL;

            self[left.into()][<u16 as Into<usize>>::into(i)] = Pixel {
                shape: [' ', BOX_DRAWINGS_LIGHT_VERTICAL],
                color: Color::Default,
            };

            self[right.into()][<u16 as Into<usize>>::into(i)] = Pixel {
                shape: [BOX_DRAWINGS_LIGHT_VERTICAL, ' '],
                color: Color::Default,
            };
        }

        use crate::unicode::{
            BOX_DRAWINGS_LIGHT_DOWN_AND_LEFT, BOX_DRAWINGS_LIGHT_DOWN_AND_RIGHT,
            BOX_DRAWINGS_LIGHT_UP_AND_LEFT, BOX_DRAWINGS_LIGHT_UP_AND_RIGHT,
        };

        // Draw the corners.
        // top left
        self[left.into()][<u16 as Into<usize>>::into(top)] = Pixel {
            shape: [' ', BOX_DRAWINGS_LIGHT_DOWN_AND_RIGHT],
            color: Color::Default,
        };

        // top right
        self[right.into()][<u16 as Into<usize>>::into(top)] = Pixel {
            shape: [BOX_DRAWINGS_LIGHT_DOWN_AND_LEFT, ' '],
            color: Color::Default,
        };

        // bottom left
        self[left.into()][<u16 as Into<usize>>::into(bottom)] = Pixel {
            shape: [' ', BOX_DRAWINGS_LIGHT_UP_AND_RIGHT],
            color: Color::Default,
        };

        // bottom right
        self[right.into()][<u16 as Into<usize>>::into(bottom)] = Pixel {
            shape: [BOX_DRAWINGS_LIGHT_UP_AND_LEFT, ' '],
            color: Color::Default,
        };

        Ok(())
    }

    pub fn draw_text(&mut self, x: u32, y: u32, text: &str) {
        if x >= self.width || y >= self.height {
            return;
        }

        text.chars()
            .collect::<Vec<char>>()
            .chunks(2)
            .enumerate()
            .for_each(|(index, characters)| {
                let pixel_x = x + index as u32;
                let pixel_y: usize = y.try_into().unwrap();

                let second_char = if let Some(second_char) = characters.get(1) {
                    *second_char
                } else {
                    ' '
                };

                if pixel_x < self.width {
                    let pixel = Pixel {
                        shape: [characters[0], second_char],
                        color: Color::Default,
                    };

                    self[pixel_x][pixel_y] = pixel;
                }
            });
    }
}

impl Index<u32> for Screen {
    type Output = [Pixel];

    // Returns a slice of the pixels which can also be indexed into. This creates
    // the illusion that the Screen is 2D when it's actually 1D.
    fn index(&self, index: u32) -> &Self::Output {
        // The start and end of the slice.
        let start: usize = (index * self.height).try_into().unwrap();
        let end: usize = ((index + 1) * self.height).try_into().unwrap();

        &self.pixels[start..end]
    }
}

impl IndexMut<u32> for Screen {
    // Same thing as the immutable index trait implementation, except that this one
    // is, well, mutable. Not sure why I have to write the same thing twice, hoping
    // that there's a way to eliminate this sort of boilerplate, but it isn't a lot
    // so I'm not going to put in the effort of setting up all the abstractions and
    // all that stuff just to get rid of a little bit of boilerplate.
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        // The start and end of the slice.
        let start: usize = (index * self.height).try_into().unwrap();
        let end: usize = ((index + 1) * self.height).try_into().unwrap();

        // By the way, the expected behaviour of the index operator is that it panics
        // if it goes out of bound, which is why I'm calling `unwrap()` here. I should
        // probably write proper error messages with `expect()`, but I'm too lazy and
        // this is a rather small project so it doesn't matter.

        &mut self.pixels[start..end]
    }
}

// Only UNIX systems needs to perform some shutdown actions.
// Windows has their own function for non-blocking input.
#[cfg(target_family = "unix")]
impl Drop for Screen {
    fn drop(&mut self) {
        unsafe {
            let mut terminal_settings = term::termios::default();
            term::tcgetattr(unistd::STDIN_FILENO as i32, &mut terminal_settings);
            terminal_settings.c_lflag |= term::ICANON;
            term::tcsetattr(
                unistd::STDIN_FILENO as i32,
                term::ICANON as i32,
                &terminal_settings,
            );
        }
    }
}
