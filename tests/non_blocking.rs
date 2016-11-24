
extern crate drm;

use drm::Device;
use std::io::ErrorKind;

#[test]
fn set_notblocking() {
    let mut dev = Device::first_card().expect("Failed to open card");
    dev.set_nonblocking(true).expect("Failed to set to non-blocking");

    match dev.read_event() {
        Err(ref err) if err.kind() == ErrorKind::WouldBlock => { /* Got correct error */ }
        x => panic!("Got {:?} expected WouldBlock", x),
    }

    dev.set_nonblocking(false).expect("Failed to set to blocking again");
}
