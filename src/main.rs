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
    
    let mut color = screen::colors::basic::RED;
    
    while running {
        if let Some(input) = Screen::read_input() {
            match input {
                'w' => if player_y < (screen.width() - 1).try_into().unwrap() { player_y += 1 },
                's' => if player_y > 0 { player_y -= 1 },
                'q' => running = false,
                _ => (),
            }
        }
        
        if color == 37 {
            color = 30;
        } else {
            color += 1;
        }

        screen.clear();

        screen[player_x][player_y].set_shape('#', '#');
        screen[player_x][player_y].set_color(screen::Color::Basic(color));
        screen.present();
    }
}
