
extern crate drm;
use drm::*;

fn main() {
    let dev = Device::first_card().unwrap();
    let version: Version = dev.version().unwrap();
    println!("{:#?}", version);
    println!("name: {}", version.name().unwrap());
    println!("date: {}", version.date().unwrap());
    println!("desc: {}", version.desc().unwrap());

    match dev.busid() {
        Ok(busid) => println!("Busid: {}", busid),
        Err(err) => panic!("Failed to get busid: {}", err),
    }
}
