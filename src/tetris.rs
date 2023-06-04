// This file contains all the logic that is related to the actual Tetris game itself.
// This includes the game mechanics, the game abstractions, etc.

use std::num::TryFromIntError;

use crate::screen::{self, OutOfBoundsError, Pixel, Screen};

const SCREEN_WIDTH: u32 = 10;
const SCREEN_HEIGHT: u32 = 20;

pub struct Tetris {
    screen: Screen,
    is_running: bool,
}

impl Tetris {
    pub fn new() -> Result<Tetris, TryFromIntError> {
        Ok(Tetris {
            screen: Screen::new(SCREEN_WIDTH, SCREEN_HEIGHT)?,
            is_running: true,
        })
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn update(&mut self) {
        if let Ok(input) = self.screen.read_input() {
            // TODO: Do stuff here
        }
    }

    pub fn render(&mut self) -> Result<(), OutOfBoundsError> {
        self.screen.clear();
        self.screen.fill_with_pixel(&Pixel {
            shape: [crate::unicode::LIGHT_SHADE, ' '],
            color: screen::Color::Basic(screen::colors::basic::BRIGHT_BLACK),
        });
        self.screen
            .draw_box(0, 0, (SCREEN_WIDTH - 2) as u16, (SCREEN_HEIGHT - 2) as u16)?;

        self.screen.present();

        Ok(())
    }
}
