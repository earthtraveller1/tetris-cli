mod screen;
mod system;

use screen::Screen;
use screen::Shape;

fn main() {
    let mut screen = Screen::new(16, 16).unwrap();

    let player_x = 8;
    let mut player_y = 8;

    let player_shape = Shape {
        pixels: vec![(0, 0), (0, 1), (0, -1), (-1, 0), (1, 0)],
        fill_pixel: screen::Pixel {
            shape: ['#', '#'],
            color: screen::Color::Basic(screen::colors::basic::GREEN),
        },
    };

    screen.draw_shape(&player_shape, player_x, player_y);
    screen.present();

    let mut running = true;
    while running {
        if let Some(input) = Screen::read_input() {
            match input {
                'w' => {
                    if player_y < (screen.width() - 1).try_into().unwrap() {
                        player_y += 1
                    }
                }
                's' => {
                    if player_y > 0 {
                        player_y -= 1
                    }
                }
                'q' => running = false,
                _ => (),
            }
        }

        screen.clear();
        screen.draw_shape(&player_shape, player_x, player_y);

        screen.present();
    }
}
