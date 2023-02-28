use std::error::Error;

// A basic representation of a "pixel"
#[derive(Clone)]
struct Pixel {
    // The shape of the pixel, as in what is printed out when the pixel is displayed.
    // It is a two-element array because every "pixel" takes up two characters on the
    // terminal.
    shape: [char; 2],
    
    // The color of the pixel
    color: Color
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            shape: [' ', ' '],
            color: Color::Default
        }
    }
}

// Support for RGB
#[derive(Clone)]
pub enum Color {
    Default,
    RGB(u8, u8, u8)
}

// A basic abstraction of a screen that makes it easier to render bitmap graphics
// on the terminal
pub struct Screen {
    width: u32,
    height: u32,

    // Used a single-dimensional vector instead of a vector of vectors to improve
    // performance.
    pixels: Vec<Pixel>
}

impl Screen {
    // Creates a blank screen with a certain width and height.
    pub fn new(width: u32, height: u32) -> Result<Screen, std::num::TryFromIntError> {
        Ok(Screen {
            width,
            height,
            pixels: vec![Pixel::default(); (width * height).try_into()?]
        })
    }
}