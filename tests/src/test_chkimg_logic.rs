use image::{GenericImageView, Rgba};

fn pixel_match(p1: Rgba<u8>, p2: Rgba<u8>, tolerance: i32) -> bool {
    if tolerance == 0 { return p1 == p2; }
    for i in 0..3 {
        let diff = (p1[i] as i32 - p2[i] as i32).abs();
        if diff > tolerance { return false; }
    }
    true
}

fn main() {
    let screen = image::open("tests/fixtures/images/failure_screen.png").unwrap().to_rgba8();
    let template = image::open("tests/fixtures/images/guaranteed_template.png").unwrap().to_rgba8();
    let (sw, sh) = screen.dimensions();
    let (tw, th) = template.dimensions();
    let tolerance = 25;
    
    let first_pixel = template.get_pixel(0, 0);
    println!("Screen size: {}x{}", sw, sh);
    println!("Template size: {}x{}", tw, th);
    println!("Searching with tolerance {}...", tolerance);

    let mut min_diff = f64::MAX;
    let mut best_pos = (0, 0);

    for y in 0..=(sh - th) {
        for x in 0..=(sw - tw) {
            let mut total_pixel_diff = 0.0;
            let mut matched = true;
            'inner: for ty in 0..th {
                for tx in 0..tw {
                    let sp = screen.get_pixel(x + tx, y + ty);
                    let tp = template.get_pixel(tx, ty);
                    let mut d = 0;
                    for i in 0..3 {
                        d += (sp[i] as i32 - tp[i] as i32).abs();
                    }
                    total_pixel_diff += d as f64;
                    if d > tolerance {
                        matched = false;
                        // break 'inner; // Don't break so we can calc total diff
                    }
                }
                if !matched { break 'inner; }
            }
            
            let avg_diff = total_pixel_diff / (tw * th) as f64;
            if avg_diff < min_diff {
                min_diff = avg_diff;
                best_pos = (x, y);
            }

            if matched {
                println!("FOUND at ({}, {})", x, y);
                return;
            }
        }
    }
    println!("NOT FOUND. Best match at ({}, {}) with avg diff {}", best_pos.0, best_pos.1, min_diff);
}
