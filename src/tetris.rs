// This file contains all the logic that is related to the actual Tetris game itself.
// This includes the game mechanics, the game abstractions, etc.

use std::num::TryFromIntError;

use crate::screen::{self, Pixel, Screen, Shape};

pub const SCREEN_WIDTH: u32 = 10;
pub const SCREEN_HEIGHT: u32 = 20;

pub const PLAYER_STARTING_X: u16 = 5;
pub const PLAYER_STARTING_Y: u16 = 3;

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

    blocks: Vec<[Option<u8>; SCREEN_WIDTH as usize]>,

    current_shape: Option<Shape>,
}

impl Tetris {
    pub fn new() -> Result<Tetris, TryFromIntError> {
        Ok(Tetris {
            screen: Screen::new(SCREEN_WIDTH + 2, SCREEN_HEIGHT + 2)?,
            is_running: true,

            random_generator: RandomGenerator::new(101, 4, 1),

            fall_timer: 0,

            player_x: PLAYER_STARTING_X,
            player_y: PLAYER_STARTING_Y,

            blocks: vec![[None; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize],

            current_shape: None, // TODO: Select random shape
        })
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    // Checks if the current shape is within the bounds of the game.
    fn is_shape_in_bounds(&self) -> (bool, bool) {
        if let Some(current_shape) = self.current_shape.as_ref() {
            let mut within_x_bounds = true;
            let mut within_y_bounds = true;

            current_shape.pixels.iter().for_each(|(block_x, block_y)| {
                let block_x: i16 =
                    block_x + <u16 as TryInto<i16>>::try_into(self.player_x).unwrap();
                let block_y: i16 =
                    block_y + <u16 as TryInto<i16>>::try_into(self.player_y).unwrap();

                if block_y >= SCREEN_HEIGHT as i16 || block_y <= 0 {
                    within_y_bounds = false;
                    return;
                }

                if block_x > SCREEN_WIDTH as i16 || block_x <= 0 {
                    within_x_bounds = false;
                    return;
                }

                // Check that it is not colliding with fossilized blocks.

                if let Some(_) = self.blocks
                    [<i16 as TryInto<usize>>::try_into(block_y - 1).unwrap()]
                    [<i16 as TryInto<usize>>::try_into(block_x - 1).unwrap()]
                {
                    within_x_bounds = false;
                    within_y_bounds = false;
                }
            });

            (within_x_bounds, within_y_bounds)
        } else {
            (false, false)
        }
    }

    fn fossilize_current_piece(&mut self) {
        if let Some(shape) = self.current_shape.as_ref() {
            shape.pixels.iter().for_each(|(component_x, component_y)| {
                self.blocks[<i16 as TryInto<usize>>::try_into(
                    *component_y + <u16 as TryInto<i16>>::try_into(self.player_y).unwrap(),
                )
                .unwrap()][<i16 as TryInto<usize>>::try_into(
                    *component_x + <u16 as TryInto<i16>>::try_into(self.player_x - 1).unwrap(),
                )
                .unwrap()] = if let crate::screen::Color::Basic(color) = shape.fill_pixel.color {
                    Some(color)
                } else {
                    None
                };
            });

            self.current_shape = None;
        }
    }

    fn fall_until_hit(&mut self) {
        loop {
            let (_, not_at_bottom) = self.is_shape_in_bounds();
            if not_at_bottom {
                self.player_y += 1;
            } else {
                self.player_y -= 1;
                self.fossilize_current_piece();

                break;
            }
        }
    }

    pub fn update(&mut self) {
        if self.fall_timer == <u8 as Into<u16>>::into(crate::FRAME_RATE) / 2 {
            self.fall_timer = 0;

            // Only fall if we are not at the bottom.
            let (_, not_at_bottom) = self.is_shape_in_bounds();
            if not_at_bottom {
                self.player_y += 1;
            } else {
                self.player_y -= 1;
                self.fossilize_current_piece();
            }
        }

        self.fall_timer += 1;

        if let Ok(input) = self.screen.read_input() {
            match input {
                'q' => self.is_running = false,
                'a' => {
                    if self.player_x > 0 {
                        self.player_x -= 1;
                        let (within_bounds, _) = self.is_shape_in_bounds();

                        if !within_bounds {
                            self.player_x += 1;
                        }
                    }
                }
                'd' => {
                    self.player_x += 1;
                    let (within_bounds, _) = self.is_shape_in_bounds();

                    if !within_bounds {
                        self.player_x -= 1;
                    }
                }
                's' => {
                    if let Some(current_shape) = self.current_shape.as_mut() {
                        current_shape.rotate(true);

                        // This is to prevent rotating the shape out of bounds.
                        let (within_x_bounds, within_y_bounds) =
                            current_shape.is_within_bounds(self.player_x, self.player_y);
                        if !within_x_bounds || !within_y_bounds {
                            // Undo the rotation if it results in the shape going out of bounds.
                            current_shape.rotate(false);
                        }
                    }
                }
                'w' => {
                    if let Some(current_shape) = self.current_shape.as_mut() {
                        current_shape.rotate(false);

                        // This is to prevent rotating the shape out of bounds.
                        let (within_x_bounds, within_y_bounds) =
                            current_shape.is_within_bounds(self.player_x, self.player_y);
                        if !within_x_bounds || !within_y_bounds {
                            // Undo the rotation if it results in the shape going out of bounds.
                            current_shape.rotate(true);
                        }
                    }
                }
                'z' => {
                    if let Some(current_shape) = self.current_shape.as_mut() {
                        current_shape.flip(true);
                    }

                    // Checks are not needed here, as it is impossible to flip out of bounds.
                }
                'x' => {
                    if let Some(current_shape) = self.current_shape.as_mut() {
                        current_shape.flip(false);
                    }

                    // Checks are not needed here, as it is impossible to flip out of bounds.
                }
                ' ' => self.fall_until_hit(),
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
                self.player_x = PLAYER_STARTING_X;
                self.player_y = PLAYER_STARTING_Y;

                self.current_shape = Some(
                    SHAPES[<u64 as TryInto<usize>>::try_into(self.random_generator.generate())
                        .unwrap()
                        % 7]
                    .clone(),
                );
                self.current_shape.as_ref().unwrap()
            }
        };

        // Render the blocks onto the screen
        self.blocks.iter().enumerate().for_each(|(i, row)| {
            for j in 0..SCREEN_WIDTH {
                if let Some(color) = row[<u32 as TryInto<usize>>::try_into(j).unwrap()] {
                    use crate::screen::Color;
                    use crate::unicode::FULL_BLOCK;

                    self.screen[j + 1][i + 1] = Pixel {
                        shape: [FULL_BLOCK, FULL_BLOCK],
                        color: Color::Basic(color),
                    };
                }
            }
        });

        self.screen
            .draw_shape(current_shape, self.player_x, self.player_y);

        self.screen.present();
    }
}
