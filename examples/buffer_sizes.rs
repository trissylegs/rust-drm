
extern crate drm;

use drm::Device;
use drm::mode::*;

fn main() {
    let device = Device::first_card().unwrap();

    println!("width = 64, height = 64");
    for bpp in 0..33 {
        for depth in 0..33 {
            print!("bpp: {}, depth: {}\t", bpp, depth);
            match DumbBuf::create_with_depth(&device, 64, 64, bpp, depth) {
                Ok(buf) => {
                    println!("Yes, size = {}, stride = {}", buf.bytes(), buf.pitch());
                }
                Err(error) => println!("No,  error = {}", error),
            }
        }
    }
}
