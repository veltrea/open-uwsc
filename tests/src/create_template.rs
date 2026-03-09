// create_template.rs
use image::{Rgba, RgbaImage};

fn main() {
    let mut img = RgbaImage::new(20, 20);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([0, 255, 0, 255]); // Solid Green
    }
    img.save("green_square.png").expect("Failed to save image");
    println!("Created green_square.png");
}
