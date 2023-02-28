use std::ops::{Add, Index, IndexMut, Mul};

// A basic representation of a "pixel"
#[derive(Clone)]
pub struct Pixel {
    // The shape of the pixel, as in what is printed out when the pixel is displayed.
    // It is a two-element array because every "pixel" takes up two characters on the
    // terminal.
    shape: [char; 2],

    // The color of the pixel
    color: Color,
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
    RGB(u8, u8, u8),
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
        Ok(Screen {
            width,
            height,
            pixels: vec![Pixel::default(); (width * height).try_into()?],
        })
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
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        self.width = new_width;
        self.height = new_height;
        self.pixels.resize(
            (new_width * new_height).try_into().unwrap(),
            Pixel::default(),
        );
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
