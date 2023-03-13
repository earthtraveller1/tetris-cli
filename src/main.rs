mod screen;
mod system;

use screen::Screen;

fn main() {
    let mut screen = Screen::new(16, 16).unwrap();

    let player_x = 8;
    let mut player_y = 8;

    screen[player_x][player_y].set_shape('#', '#');
    screen.present();

    let mut running = true;
    
    while running {
        if let Some(input) = Screen::read_input() {
            match input {
                'w' => player_y += 1,
                's' => player_y -= 1,
                'q' => running = false,
                _ => (),
            }
        }

        screen.clear();

        screen[player_x][player_y].set_shape('#', '#');
        screen.present();
    }
}
