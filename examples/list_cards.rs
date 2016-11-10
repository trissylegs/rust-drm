
extern crate drm;
use drm::Device;
use std::fs::Metadata;
use std::os::unix::prelude::*;

fn main() {
    for path in Device::cards().unwrap() {
        let mut dev = Device::open(&path).unwrap();
        println!("{}", path.display());
        println!(" driver info:");
        let version = dev.version().unwrap();
        println!("  version {}.{}.{}",
                 version.number().0, version.number().1, version.number().2);
        println!("  name: {}", version.name().unwrap());
        let date = version.date().unwrap();
        println!("  date: {}-{}-{}", &date[0..4], &date[4..6], &date[6..]);
        println!("  {}", version.desc().unwrap());
    }
}
