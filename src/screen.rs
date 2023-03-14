// This file probably shouldn't be called "screen" since it contains a lot more than
// just the screen abstraction. It also contains the rendering logic.

#[cfg(target_family = "unix")]
use super::system::{termios as term, unistd};
use std::ops::{Add, Index, IndexMut, Mul};

// A basic representation of a "pixel"
#[derive(Clone)]
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
#[derive(Clone)]
pub enum Color {
    Default,
    Basic(u8), // Basic color support. Use for maximum compatibility. Only have 16 colors available.
    RGB(u8, u8, u8),
}

pub mod colors {
    pub mod basic {
        pub const BLACK: u8 = 30;
        pub const RED: u8 = 31;
        pub const GREEN: u8 = 32;
        pub const YELLOW: u8 = 33;
        pub const BLUE: u8 = 34;
        pub const MAGENTA: u8 = 35;
        pub const CYAN: u8 = 36;
        pub const WHITE: u8 = 37;

        // Bright colors.
        pub const BRIGH_BLACK: u8 = 90;
        pub const BRIGHT_RED: u8 = 91;
        pub const BRIGHT_GREEN: u8 = 92;
        pub const BRIGHT_YELLOW: u8 = 93;
        pub const BRIGHT_BLUE: u8 = 94;
        pub const BRIGHT_MAGENTA: u8 = 95;
        pub const BRIGHT_CYAN: u8 = 96;
        pub const BRIGHT_WHITE: u8 = 97;
    }
}

// A basic abstraction of a screen that makes it easier to render bitmap graphics
// on the terminal
pub struct Screen {
    width: u32,
    height: u32,

    // Used a single-dimensional vector instead of a vector of vectors to improve
    // performance.
    pixels: Vec<Pixel>,
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
            term::tcsetattr(
                unistd::STDIN_FILENO as i32,
                term::TCSANOW as i32,
                &terminal_settings,
            );
        }

        Ok(Screen {
            width,
            height,
            pixels: vec![Pixel::default(); (width * height).try_into()?],
        })
    }
    
    // Returns the width of the screen. This can be used by clients to ensure
    // that they don't try to write to pixels that are out of bounds, which
    // can cause the program to panic.
    pub fn width(&self) -> u32 { self.width }
    
    // Returns the height of the screen. Same use case as the width() function
    pub fn height(&self) -> u32 { self.height }

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

    // Basically, read whatever key the user has pressed from the terminal
    // This is the UNIX version. The Windows version uses Microsoft's dedicated
    // function instead of getchar.
    #[cfg(target_family = "unix")]
    pub fn read_input() -> Option<char> {
        unsafe { char::from_u32(crate::system::getchar().try_into().ok()?) }
    }
    
    // The Windows version of read input. Basically does the exact same
    // thing, but for windows.
    #[cfg(target_family = "windows")]
    pub fn read_input() -> Option<char> {
        unsafe { char::from_u32(crate::system::conio::_getch().try_into().ok()?) }
    }

    // Finally, the function that you've all been waiting for. This guy does all of the
    // hard work of going through the pixels and drawing them on the terminal.
    pub fn present(&self) {
        // Move to the start of the screen before printing.
        print!("\x1B[H");

        for i in 0..self.height {
            for j in 0..self.width {
                let pixel: &Pixel = &self[i][j as usize];

                // I'm sorry that this is way too hard to read but basically it's text
                // colors that supports RGB. I don't have time to explain this but I can
                // give you the link if you would like.
                //
                // https://en.wikipedia.org/wiki/ANSI_escape_code#24-bit
                if let Color::RGB(red, green, blue) = pixel.color {
                    print!(
                        "\x1B[38;2;{};{};{}m{}{}\x1B[0m",
                        red, green, blue, pixel.shape[0], pixel.shape[1]
                    )
                } else if let Color::Basic(code) = pixel.color {
                    print!("\x1B[{}m{}{}\x1B[0m", code, pixel.shape[0], pixel.shape[1]);
                } else {
                    print!("{}{}", pixel.shape[0], pixel.shape[1]);
                }
            }

            println!("");
        }
    }
}

// A struct for a shape.
pub struct Shape {
    // The squares that are taken up by the shape, relative to the
    // shape itself.
    pub pixels: Vec<(i16, i16)>,
    // The pixel to fill the shape with.
    pub fill_pixel: Pixel
}

// Alright, so for the purpose of organization, I'm going to put
// the high-level rendering logics into a different implementation
// block.
impl Screen {
    // Draws a shape.
    pub fn draw_shape(&mut self, shape: &Shape, x_pos: u16, y_pos: u16) {
        let x_pos: i16 = x_pos.try_into().unwrap();
        let y_pos: i16 = y_pos.try_into().unwrap();

        shape.pixels.iter().for_each(|(pixel_x, pixel_y)| {
            let real_x: i16 = x_pos + *pixel_x;
            let real_y: i16 = y_pos + *pixel_y;

            // Any pixels that are out of bounds are automatically clipped off.
            // Also, the casting is safe as the || operators are short-circuited.
            if (real_x < 0 || real_x as u32 >= self.width) || (real_y < 0 || real_y as u32 >= self.height) {
                return;
            }
            
            // Both u32 and usize are larger than i16 and real_x and real_y are both 
            // guaranteed to be positive by this point, so this is safe (at least it
            // should be).
            self[real_x as u32][real_y as usize] = shape.fill_pixel.clone();
        })
    }
    
    // Fills the screen with a specific color.
    pub fn fill_with_pixel(&mut self, pixel: &Pixel) {
        for i in 0..self.width {
            for j in 0..self.height {
                self[i][j as usize] = pixel.clone();
            }
        }
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