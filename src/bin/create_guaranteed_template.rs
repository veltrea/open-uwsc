// create_guaranteed_template.rs
use image::{GenericImageView, ImageBuffer, Rgba};
use screenshots::Screen;

fn main() {
    let screens = Screen::all().unwrap();
    let screen = screens[0];
    let image = screen.capture().unwrap();
    
    // Crop a 50x50 area from (100, 100)
    let cropped = image.view(100, 100, 50, 50).to_image();
    cropped.save("guaranteed_template.png").expect("Failed to save template");
    println!("Created guaranteed_template.png from screen coordinates (100, 100)");
}
