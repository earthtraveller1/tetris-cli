mod screen;

use screen::Screen;

fn main() {
    let mut screen = Screen::new(12, 12).unwrap();
    for i in 0..12 {
        for j in 0..12 {
            let pixel = &mut screen[i][j as usize];
            
            pixel.set_shape('$', '$');
            
            if i == j {
                pixel.set_color(screen::Color::RGB(255, 0, 255));
            }
        }
    }
    screen.present();
}
