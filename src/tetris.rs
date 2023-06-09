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
    modulus: u64,
    multiplier: u64,
    increment: u64,
    seed: u64,
}

impl RandomGenerator {
    fn new(modulus: u64, multiplier: u64, increment: u64) -> RandomGenerator {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now();
        let seed = now.duration_since(UNIX_EPOCH).unwrap().as_secs();

        RandomGenerator {
            modulus,
            multiplier,
            increment,
            seed,
        }
    }

    fn generate(&mut self) -> u64 {
        let result = (self.multiplier * self.seed + self.increment) % self.modulus;
        self.seed = result;
        result
    }
}

pub struct Tetris {
    screen: Screen,
    is_running: bool,

    random_generator: RandomGenerator,

    // This value is decremented every frame, and when it reaches the value of the framerate
    // , it will be resetted back to zero and the playing piece will fall one unit down.
    fall_timer: u16,

    player_x: u16,
    player_y: u16,

    current_shape: Option<Shape>,
}

impl Tetris {
    pub fn new() -> Result<Tetris, TryFromIntError> {
        Ok(Tetris {
            screen: Screen::new(SCREEN_WIDTH + 2, SCREEN_HEIGHT + 2)?,
            is_running: true,
            random_generator: RandomGenerator::new(101, 4, 1),
            fall_timer: 0,
            player_x: 5,
            player_y: 3,
            current_shape: None, // TODO: Select random shape
        })
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn update(&mut self) {
        if self.fall_timer == crate::FRAME_RATE.into() {
            self.fall_timer = 0;
            self.player_y += 1;
        }

        self.fall_timer += 1;

        if let Ok(input) = self.screen.read_input() {
            match input {
                'q' => self.is_running = false,
                'w' => {
                    self.current_shape = None;
                }
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
                self.current_shape = Some(
                    SHAPES[<u64 as TryInto<usize>>::try_into(self.random_generator.generate())
                        .unwrap()
                        % 7]
                    .clone(),
                );
                self.current_shape.as_ref().unwrap()
            }
        };

        self.screen
            .draw_shape(current_shape, self.player_x, self.player_y);

        self.screen.present();
    }
}
