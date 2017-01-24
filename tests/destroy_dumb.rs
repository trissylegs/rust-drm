
extern crate drm;
use drm::Device;
use drm::mode::*;

#[test]
fn destroy_dumb() {
    let dev = Device::first_card().expect("Failed to open card");
    let buf = DumbBuf::<u32>::create(&dev, 640, 480).expect("Failed to crate dumbbuf");

    drop(buf);

    println!("Didn't panic!");
}
