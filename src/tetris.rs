// This file contains all the logic that is related to the actual Tetris game itself.
// This includes the game mechanics, the game abstractions, etc.

use std::num::TryFromIntError;

use crate::screen::{self, OutOfBoundsError, Pixel, Screen, Shape};

const SCREEN_WIDTH: u32 = 10;
const SCREEN_HEIGHT: u32 = 20;

mod shapes {
    use crate::{
        screen::{colors::basic::*, Color},
        tetris::{Pixel, Shape},
        unicode::FULL_BLOCK,
    };

    pub static SQUARE: Shape = Shape {
        pixels: [(0, 0), (1, 0), (1, 1), (0, 1)],
        fill_pixel: Pixel {
            shape: [FULL_BLOCK, FULL_BLOCK],
            color: Color::Basic(YELLOW),
        },
    };

    pub static STRAIGHT: Shape = Shape {
        pixels: [(0, -1), (0, 0), (0, 1), (0, 2)],
        fill_pixel: Pixel {
            shape: [FULL_BLOCK, FULL_BLOCK],
            color: Color::Basic(CYAN),
        },
    };

    pub static LEFT_SKEWED: Shape = Shape {
        pixels: [(-1, 0), (0, 0), (0, -1), (1, -1)],
        fill_pixel: Pixel {
            shape: [FULL_BLOCK, FULL_BLOCK],
            color: Color::Basic(GREEN),
        },
    };

    pub static RIGHT_SKEWED: Shape = Shape {
        pixels: [(-1, -1), (0, 0), (0, -1), (1, 1)],
        fill_pixel: Pixel {
            shape: [FULL_BLOCK, FULL_BLOCK],
            color: Color::Basic(GREEN),
        },
    };
}

pub struct Tetris {
    screen: Screen,
    is_running: bool,

    current_shape: Option<Shape>,
}

impl Tetris {
    pub fn new() -> Result<Tetris, TryFromIntError> {
        Ok(Tetris {
            screen: Screen::new(SCREEN_WIDTH + 2, SCREEN_HEIGHT + 2)?,
            is_running: true,
            current_shape: None, // TODO: Select random shape
        })
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn update(&mut self) {
        if let Ok(input) = self.screen.read_input() {
            match input {
                'q' => self.is_running = false,
                _ => (),
            }
        }
    }

    pub fn render(&mut self) -> Result<(), OutOfBoundsError> {
        self.screen.clear();
        self.screen.fill_with_pixel(&Pixel {
            shape: [crate::unicode::LIGHT_SHADE, ' '],
            color: screen::Color::Basic(screen::colors::basic::BRIGHT_BLACK),
        });

        self.screen
            .draw_box(0, 0, (SCREEN_WIDTH + 1) as u16, (SCREEN_HEIGHT + 1) as u16)?;

        let current_shape = match self.current_shape.as_ref() {
            Some(shape) => shape,
            None => {
                self.current_shape = Some(shapes::SQUARE.clone()); // TODO: Random shape selection.
                self.current_shape.as_ref().unwrap()
            }
        };

        self.screen.draw_shape(current_shape, 5, 5);

        self.screen.present();

        Ok(())
    }
}
