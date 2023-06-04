// This file contains all the logic that is related to the actual Tetris game itself.
// This includes the game mechanics, the game abstractions, etc.

use std::num::TryFromIntError;

use crate::screen::{self, OutOfBoundsError, Pixel, Screen, Shape};

const SCREEN_WIDTH: u32 = 10;
const SCREEN_HEIGHT: u32 = 20;

struct Shapes {
    square: Shape,
}

impl Shapes {
    fn new() -> Shapes {
        use crate::screen::{colors::basic::*, Color};
        use crate::unicode::FULL_BLOCK;

        Shapes {
            square: Shape {
                pixels: vec![(0, 0), (1, 0), (1, 1), (0, 1)],
                fill_pixel: Pixel {
                    shape: [FULL_BLOCK, FULL_BLOCK],
                    color: Color::Basic(YELLOW),
                },
            },
        }
    }
}

pub struct Tetris {
    screen: Screen,
    is_running: bool,

    shapes: Shapes,
}

impl Tetris {
    pub fn new() -> Result<Tetris, TryFromIntError> {
        Ok(Tetris {
            screen: Screen::new(SCREEN_WIDTH + 2, SCREEN_HEIGHT + 2)?,
            is_running: true,
            shapes: Shapes::new(),
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

        self.screen.draw_shape(&self.shapes.square, 5, 5);

        self.screen.present();

        Ok(())
    }
}
