// This file contains all the logic that is related to the actual Tetris game itself.
// This includes the game mechanics, the game abstractions, etc.

use std::num::TryFromIntError;

use crate::screen::{self, Pixel, Screen, Shape};

pub const GAME_WIDTH: u32 = 10;
pub const GAME_HEIGHT: u32 = 20;

pub const SCREEN_HEIGHT: u32 = 25;
pub const SCREEN_WIDTH: u32 = 30;

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

    // This value is incremented every frame, and when it reaches the value of the framerate
    // , it will be resetted back to zero and the playing piece will fall one unit down.
    fall_timer: u16,
    // The rate at which the fall timer will be decremented per tick.
    fall_speed: f32,

    player_x: u16,
    player_y: u16,

    score: u32,

    blocks: Vec<[Option<u8>; GAME_WIDTH as usize]>,

    previous_shape: Option<Shape>,
    current_shape: Option<Shape>,
    held_shape: Option<Shape>,

    can_hold_shape: bool,
}

impl Tetris {
    pub fn new() -> Result<Tetris, TryFromIntError> {
        Ok(Tetris {
            screen: Screen::new(SCREEN_WIDTH, SCREEN_HEIGHT)?,
            is_running: true,

            random_generator: RandomGenerator::new(101, 4, 1),

            fall_timer: 0,
            fall_speed: 1.0,

            player_x: PLAYER_STARTING_X,
            player_y: PLAYER_STARTING_Y,

            score: 0,

            blocks: vec![[None; GAME_WIDTH as usize]; GAME_HEIGHT as usize],

            previous_shape: None,
            current_shape: None, // TODO: Select random shape
            held_shape: None,
            can_hold_shape: true,
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

                if block_y >= GAME_HEIGHT as i16 || block_y <= 0 {
                    within_y_bounds = false;
                    return;
                }

                if block_x > GAME_WIDTH as i16 || block_x <= 0 {
                    within_x_bounds = false;
                    return;
                }

                // Check that it is not colliding with fossilized blocks.

                if let Some(_) = self.blocks[<i16 as TryInto<usize>>::try_into(block_y).unwrap()]
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

            self.previous_shape = self.current_shape.take();

            let mut rows_cleared = 0;

            // Now, iterate through the rows and clear the ones that are full.
            let mut i = 0;
            while i < self.blocks.len() {
                let is_row_not_full =
                    self.blocks[i]
                        .iter()
                        .find(|block| if let None = block { true } else { false });

                if let None = is_row_not_full {
                    self.blocks.remove(i);
                    self.blocks.insert(0, [None; 10]);
                    rows_cleared += 1;
                    continue;
                }

                i += 1;
            }

            self.score += rows_cleared * 100;

            if rows_cleared > 0 {
                self.score += (rows_cleared - 1) * 25
            }

            self.fall_speed += 0.1 * rows_cleared as f32;
        }

        self.can_hold_shape = true;
    }

    fn fall_until_hit(&mut self) {
        loop {
            let (_, not_at_bottom) = self.is_shape_in_bounds();
            if not_at_bottom {
                self.player_y += 1;
            } else {
                self.player_y -= 1;
                break;
            }
        }
    }

    pub fn update(&mut self) {
        if self.fall_timer >= <u8 as Into<u16>>::into(crate::FRAME_RATE) / 2 {
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

        self.fall_timer += self.fall_speed as u16;

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
                        current_shape.rotate(true);
                        current_shape.rotate(true);

                        // false -> right
                        // true -> left

                        // This is to prevent rotating the shape out of bounds.
                        let (within_x_bounds, within_y_bounds) =
                            current_shape.is_within_bounds(self.player_x, self.player_y);
                        if !within_x_bounds || !within_y_bounds {
                            // Undo the rotation if it results in the shape going out of bounds.
                            current_shape.rotate(false);
                        }
                    }

                    // Checks are not needed here, as it is impossible to flip out of bounds.
                }
                'x' => {
                    if let Some(current_shape) = self.current_shape.as_mut() {
                        current_shape.rotate(false);
                        current_shape.rotate(false);

                        // This is to prevent rotating the shape out of bounds.
                        let (within_x_bounds, within_y_bounds) =
                            current_shape.is_within_bounds(self.player_x, self.player_y);
                        if !within_x_bounds || !within_y_bounds {
                            // Undo the rotation if it results in the shape going out of bounds.
                            current_shape.rotate(true);
                        }
                    }

                    // Checks are not needed here, as it is impossible to flip out of bounds.
                }
                'h' => {
                    if self.can_hold_shape {
                        let current_shape = self.current_shape.take();
                        self.current_shape = self.held_shape.take();
                        self.previous_shape = current_shape.clone();
                        self.held_shape = current_shape;

                        self.player_x = PLAYER_STARTING_X;
                        self.player_y = PLAYER_STARTING_Y;

                        self.can_hold_shape = false;
                    }
                }
                ' ' => {
                    self.fall_until_hit();
                    self.fossilize_current_piece();
                }
                _ => (),
            }
        }
    }

    pub fn render(&mut self) {
        self.screen.clear();
        /*self.screen.fill_area_with_pixel(
            &Pixel {
                shape: [crate::unicode::LIGHT_SHADE, ' '],
                color: screen::Color::Basic(screen::colors::basic::BRIGHT_BLACK),
            },
            1,
            1,
            1 + 10,
            1 + 20,
        );*/

        let saved_y = self.player_y;

        // Render the ghost piece
        self.fall_until_hit();
        if let Some(current_shape) = self.current_shape.as_ref() {
            self.screen.draw_shape(
                current_shape,
                self.player_x,
                // The y has to be offset because there are borders surroun-
                // ding the screen, which will push everything down one uni-
                // t.
                self.player_y + 1,
                true,
            );
        }

        self.player_y = saved_y;

        self.screen
            .draw_box(0, 0, (GAME_WIDTH + 1) as u16, (GAME_HEIGHT + 1) as u16)
            .unwrap();

        self.screen.draw_text(GAME_WIDTH + 2, 1, "SCORE");
        self.screen
            .draw_text(GAME_WIDTH + 2, 2, &format!("{}", self.score));

        self.screen.draw_text(GAME_WIDTH + 2, 4, "CONTROLS");
        self.screen.draw_text(GAME_WIDTH + 2, 5, "a => Move Left");
        self.screen.draw_text(GAME_WIDTH + 2, 6, "d => Move Right");
        self.screen
            .draw_text(GAME_WIDTH + 2, 7, "w => Rotate Right");
        self.screen.draw_text(GAME_WIDTH + 2, 8, "s => Rotate Left");
        self.screen
            .draw_text(GAME_WIDTH + 2, 9, "z => Rotate left 180 degrees");
        self.screen
            .draw_text(GAME_WIDTH + 2, 10, "x => Rotate right 180 degrees");
        self.screen.draw_text(GAME_WIDTH + 2, 11, "h => Hold");
        self.screen.draw_text(GAME_WIDTH + 2, 12, "[SPACE] => Drop");

        let hold_box_x = (GAME_WIDTH + 2) as u16;
        let hold_box_y = 13;
        let hold_box_width = (GAME_HEIGHT - 13) as u16;
        let hold_box_height = (GAME_HEIGHT - 13) as u16;

        self.screen
            .draw_box(hold_box_x, hold_box_y, hold_box_width, hold_box_height)
            .unwrap();

        if let Some(held_shape) = self.held_shape.as_ref() {
            self.screen
                .draw_shape(&held_shape, hold_box_x + 4, hold_box_y + 4, false);
        }

        let current_shape = match self.current_shape.as_ref() {
            Some(shape) => shape,
            None => {
                self.player_x = PLAYER_STARTING_X;
                self.player_y = PLAYER_STARTING_Y;

                self.current_shape = {
                    loop {
                        let generated_shape = SHAPES[<u64 as TryInto<usize>>::try_into(
                            self.random_generator.generate(),
                        )
                        .unwrap()
                            % 7]
                        .clone();

                        if let Some(previous_shape) = self.previous_shape.as_ref() {
                            if generated_shape != *previous_shape {
                                break Some(generated_shape);
                            }
                        } else {
                            break Some(generated_shape);
                        }
                    }
                };

                // If the current shape is out of bounds as soon as it's spawned, then it's likely
                // because the player has lost.
                let (within_x_bounds, within_y_bounds) = self.is_shape_in_bounds();
                if !within_x_bounds || !within_y_bounds {
                    self.is_running = false;
                    return;
                }

                self.current_shape.as_ref().unwrap()
            }
        };

        // Render the blocks onto the screen
        self.blocks.iter().enumerate().for_each(|(i, row)| {
            for j in 0..GAME_WIDTH {
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
            .draw_shape(current_shape, self.player_x, self.player_y, false);

        self.screen.present();
    }
}
