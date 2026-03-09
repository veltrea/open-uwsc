use screenshots::Screen;

fn main() {
    let screens = Screen::all().unwrap();
    let screen = screens[0];
    let image = screen.capture().unwrap();
    let (w, h) = image.dimensions();
    println!("Capture size: {}x{}", w, h);
    
    // Sample some pixels
    for y in (0..h).step_by(h as usize / 10) {
        for x in (0..w).step_by(w as usize / 10) {
            let p = image.get_pixel(x, y);
            println!("Pixel at ({}, {}): [R={}, G={}, B={}, A={}]", x, y, p[0], p[1], p[2], p[3]);
        }
    }
}
