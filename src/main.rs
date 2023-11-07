mod screen;
mod system;
mod tetris;

use std::thread;
use std::time::Duration;
use std::time::Instant;

const FRAME_RATE: u8 = 60;

// Unicode literals that might be useful in future.
mod unicode {
    pub const FULL_BLOCK: char = '\u{2588}';
    pub const LIGHT_SHADE: char = '\u{2591}';

    // Basic box drawing.
    pub const BOX_DRAWINGS_LIGHT_HORIZONTAL: char = '\u{2500}';
    pub const BOX_DRAWINGS_LIGHT_VERTICAL: char = '\u{2502}';

    // Box drawing corners.
    pub const BOX_DRAWINGS_LIGHT_DOWN_AND_RIGHT: char = '\u{250C}';
    pub const BOX_DRAWINGS_LIGHT_DOWN_AND_LEFT: char = '\u{2510}';
    pub const BOX_DRAWINGS_LIGHT_UP_AND_RIGHT: char = '\u{2514}';
    pub const BOX_DRAWINGS_LIGHT_UP_AND_LEFT: char = '\u{2518}';
}

fn main() {
    let mut game = tetris::Tetris::new().expect("Uh oh");

    while game.is_running() {
        let start = Instant::now();

        game.update();
        game.render();

        let elapsed_time = start.elapsed();

        // If the uncapped framerate is less than 30, then we simply
        // leave it be. There's nothing we can do about that here.
        if elapsed_time.as_millis() > 1000 / FRAME_RATE as u128 {
            continue;
        }

        let wait_duration = Duration::from_millis(1000 / FRAME_RATE as u64) - elapsed_time;
        thread::sleep(wait_duration);
    }
}
