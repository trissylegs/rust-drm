
extern crate drm;

use drm::Device;
use drm::Event;

fn main()
{
    let mut dev = Device::first_card().expect("Failed to get card");

    let mut prev_tv = None;
    loop {
        // Request a vblank event.
        dev.request_vblank(0, 0).expect("Failed to request id");
        // Wait for the event.
        let ev = dev.read_event().expect("Failed to read event");
        print!("{:?}\t", ev);
        
        match (prev_tv, ev) {
            (Some(prev), Event::VBlank { tv, .. }) => {
                print!("Difference: {:?}", tv.duration_since(prev));
                prev_tv = Some(tv)
            },
            (None, Event::VBlank {tv, ..}) => {
                prev_tv = Some(tv)
            }
            _ => (),
        }
        
        println!("");
    }
}
