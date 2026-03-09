use image::GenericImageView;

fn main() {
    let img = image::open("guaranteed_template.png").unwrap();
    let p = img.get_pixel(0, 0);
    println!("Template pixel (0,0): [R={}, G={}, B={}, A={}]", p[0], p[1], p[2], p[3]);
    let (w, h) = img.dimensions();
    println!("Template size: {}x{}", w, h);
}
