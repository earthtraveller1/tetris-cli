mod screen;
mod system;
mod tetris;

use screen::Screen;
use screen::Shape;

use std::thread;
use std::time::Duration;
use std::time::Instant;

// Unicode literals that might be useful in future.
mod unicode {
    pub const FULL_BLOCK: char = '\u{2588}';
    pub const LIGHT_SHADE: char = '\u{2591}';

    // Basic box drawing.
    pub const BOX_DRAWINGS_LIGHT_HORIZONTAL: char = '\u{2500}';
    pub const BOX_DRAWINGS_LIGHT_VERTICAL: char = '\u{2502}';

    // Box drawing corners
    pub const BOX_DRAWINGS_LIGHT_DOWN_AND_RIGHT: char = '\u{250C}';
    pub const BOX_DRAWINGS_LIGHT_DOWN_AND_LEFT: char = '\u{2510}';
    pub const BOX_DRAWINGS_LIGHT_UP_AND_RIGHT: char = '\u{2514}';
    pub const BOX_DRAWINGS_LIGHT_UP_AND_LEFT: char = '\u{2518}';
}

struct Game {
    screen: Screen,
    running: bool,

    player_shape: Shape,

    player_x: u16,
    player_y: u16,
}

impl Game {
    fn new() -> Game {
        let screen = Screen::new(16, 16).unwrap();


        Game {
            screen,
            player_shape: Shape {
                pixels: vec![(0, 0), (0, 1), (0, -1), (-1, 0), (1, 0)],
                fill_pixel: screen::Pixel {
                    shape: [unicode::FULL_BLOCK, unicode::FULL_BLOCK],
                    color: screen::Color::Basic(screen::colors::basic::GREEN),
                },
            },
            running: true,
            player_x: 8,
            player_y: 8,
        }
    }

    fn update(&mut self) {
        if let Ok(input) = self.screen.read_input() {
            match input {
                'w' => {
                    if self.player_y < (self.screen.width() - 1).try_into().unwrap() {
                        self.player_y += 1
                    }
                }
                's' => {
                    if self.player_y > 0 {
                        self.player_y -= 1
                    }
                }
                'q' => self.running = false,
                _ => (),
            }
        }
    }

    fn render(&mut self) {
        self.screen.clear();
        self.screen.fill_with_pixel(&screen::Pixel {
            shape: [unicode::LIGHT_SHADE, ' '],
            color: screen::Color::Basic(screen::colors::basic::BRIGH_BLACK),
        });
        self.screen.draw_box(0, 0, 8, 8);
        self.screen
            .draw_shape(&self.player_shape, self.player_x, self.player_y);

        self.screen.present();
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

fn main() {
    let mut game = Game::new();

    // Render the first frame so that the user isn't staring at a blank
    // screen on startup.
    game.render();

    while game.is_running() {
        let start = Instant::now();

        game.update();
        game.render();

        let elapsed_time = start.elapsed();

        // If the uncapped framerate is less than 60, then we simply
        // leave it be. There's nothing we can do about that here.
        if elapsed_time.as_millis() > 1000 / 30 {
            continue;
        }

        let wait_duration = Duration::from_millis(1000 / 30) - elapsed_time;
        thread::sleep(wait_duration);
    }
}
