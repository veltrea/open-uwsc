use screenshots::Screen;
use image::GenericImageView;

fn pixel_match(p1: [u8; 4], p2: [u8; 4], tolerance: i32) -> bool {
    for i in 0..3 {
        if (p1[i] as i32 - p2[i] as i32).abs() > tolerance { return false; }
    }
    true
}

fn main() {
    let screens = Screen::all().unwrap();
    let screen = screens[0];
    let screen_image = screen.capture().unwrap();
    let (sw, sh) = screen_image.dimensions();
    
    // Crop a 50x50 area from the center
    let tx = sw / 2;
    let ty = sh / 2;
    let tw = 50;
    let th = 50;
    
    let template = screen_image.view(tx, ty, tw, th).to_image();
    template.save("tests/fixtures/images/self_template.png").unwrap();
    println!("Captured 50x50 template from ({}, {})", tx, ty);
    
    let first_pixel = template.get_pixel(0, 0).0;
    let tolerance = 0; // Should be EXACT match
    
    for y in 0..=(sh - th) {
        for x in 0..=(sw - tw) {
            let sp = screen_image.get_pixel(x, y).0;
            if !pixel_match(sp, first_pixel, tolerance) { continue; }
            
            let mut matched = true;
            'inner: for dty in 0..th {
                for dtx in 0..tw {
                    let sp = screen_image.get_pixel(x + dtx, y + dty).0;
                    let tp = template.get_pixel(dtx, dty).0;
                    if !pixel_match(sp, tp, tolerance) {
                        matched = false;
                        break 'inner;
                    }
                }
            }
            if matched {
                println!("SUCCESS: Found at ({}, {}) (Expected: {}, {})", x, y, tx, ty);
                return;
            }
        }
    }
    println!("FAILURE: Not found even in self-capture!");
}
