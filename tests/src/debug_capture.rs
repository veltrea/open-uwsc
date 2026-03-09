use screenshots::Screen;
use std::fs::File;

fn main() {
    let screens = Screen::all().unwrap();
    for (i, screen) in screens.iter().enumerate() {
        let image = screen.capture().unwrap();
        let filename = format!("debug_screen_{}.png", i);
        image.save(&filename).unwrap();
        println!("Saved screen {} to {}", i, filename);
        let d = screen.display_info;
        println!("Screen {}: x={}, y={}, w={}, h={}, scale={}", 
            i, d.x, d.y, d.width, d.height, d.scale_factor);
    }
}
