// This file contains all the logic that is related to the actual Tetris game itself.
// This includes the game mechanics, the game abstractions, etc.

use std::num::TryFromIntError;

use crate::screen::{self, Pixel, Screen, Shape};

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
            color: Color::Basic(BRIGHT_YELLOW),
        },
    };

    pub static STRAIGHT: Shape = Shape {
        pixels: [(0, -1), (0, 0), (0, 1), (0, 2)],
        fill_pixel: Pixel {
            shape: [FULL_BLOCK, FULL_BLOCK],
            color: Color::Basic(CYAN),
        },
    };

    pub static TEE: Shape = Shape {
        pixels: [(0, -1), (0, 0), (-1, 0), (1, 0)],
        fill_pixel: Pixel {
            shape: [FULL_BLOCK, FULL_BLOCK],
            color: Color::Basic(MAGENTA),
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
        pixels: [(-1, -1), (0, 0), (0, -1), (1, 0)],
        fill_pixel: Pixel {
            shape: [FULL_BLOCK, FULL_BLOCK],
            color: Color::Basic(RED),
        },
    };

    pub static LEFT_L: Shape = Shape {
        pixels: [(-1, 0), (0, 0), (0, -1), (0, -2)],
        fill_pixel: Pixel {
            shape: [FULL_BLOCK, FULL_BLOCK],
            color: Color::Basic(BLUE),
        },
    };

    pub static RIGHT_L: Shape = Shape {
        pixels: [(1, 0), (0, 0), (0, -1), (0, -2)],
        fill_pixel: Pixel {
            shape: [FULL_BLOCK, FULL_BLOCK],
            color: Color::Basic(YELLOW),
        },
    };
}

static SHAPES: [&Shape; 7] = [
    &shapes::SQUARE,
    &shapes::STRAIGHT,
    &shapes::TEE,
    &shapes::LEFT_SKEWED,
    &shapes::RIGHT_SKEWED,
    &shapes::LEFT_L,
    &shapes::RIGHT_L,
];

// A Pseudorandom number generator, used to decide what piece to use next.
struct RandomGenerator {
    modulus: u16,
    multiplier: u16,
    increment: u16,
    seed: u16,
}

impl RandomGenerator {
    fn new(modulus: u16, multiplier: u16, increment: u16, seed: u16) -> RandomGenerator {
        RandomGenerator {
            modulus,
            multiplier,
            increment,
            seed,
        }
    }

    fn generate(&mut self) -> u16 {
        let result = (self.multiplier * self.seed + self.increment) % self.modulus;
        self.seed = result;
        result
    }
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

    pub fn render(&mut self) {
        self.screen.clear();
        self.screen.fill_with_pixel(&Pixel {
            shape: [crate::unicode::LIGHT_SHADE, ' '],
            color: screen::Color::Basic(screen::colors::basic::BRIGHT_BLACK),
        });

        self.screen
            .draw_box(0, 0, (SCREEN_WIDTH + 1) as u16, (SCREEN_HEIGHT + 1) as u16)
            .unwrap();

        let current_shape = match self.current_shape.as_ref() {
            Some(shape) => shape,
            None => {
                self.current_shape = Some(shapes::SQUARE.clone()); // TODO: Random shape selection.
                self.current_shape.as_ref().unwrap()
            }
        };

        self.screen.draw_shape(current_shape, 2, 2);

        // Draw all the shapes for testing.
        self.screen.draw_shape(&shapes::TEE, 2, 7);
        self.screen.draw_shape(&shapes::STRAIGHT, 2, 12);

        self.screen.draw_shape(&shapes::LEFT_SKEWED, 2, 18);
        self.screen.draw_shape(&shapes::RIGHT_SKEWED, 6, 3);

        self.screen.draw_shape(&shapes::LEFT_L, 6, 8);
        self.screen.draw_shape(&shapes::RIGHT_L, 6, 14);

        self.screen.present();
    }
}
