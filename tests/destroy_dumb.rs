
extern crate drm;
use drm::Device;
use drm::mode::*;
use drm::fourcc::*;

#[test]
fn destroy_dumb() {
    let dev = Device::first_card().expect("Failed to open card");
    let buf = DumbBufOptions::new(&dev)
        .width(640).height(480)
        .format(FourCC::XRGB8888)
        .create()
        .expect("Failed to crate dumbbuf");

    drop(buf);

    println!("Didn't panic!");
}
