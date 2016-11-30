
#![feature(conservative_impl_trait)]

extern crate tokio_core;
extern crate drm;
extern crate futures;

use tokio_core::reactor::Handle;
use tokio_core::reactor::Timeout;
use futures::Future;
use drm::tokio::EventFuture;
use std::time::Duration;

fn step(handle: Handle, dev: EventFuture) -> impl Future<Item=(), Error=()> {
    dev .map(move |(mut dev, blank)| {
            println!("{:?}", blank);
            dev.get_mut().request_vblank(0, 0).unwrap();
            handle.clone().spawn(step(handle, dev));
        })
        .map_err(|err| {
            println!("Error: {}", err);
        })
}

fn main() {
    let mut dev = drm::Device::first_card().unwrap();
    dev.set_nonblocking(true).unwrap();
    
    let mut core = tokio_core::reactor::Core::new().unwrap();
    
    let handle = core.handle();
    dev.request_vblank(0, 0).unwrap();
    
    let events = step(core.handle(), dev.event_stream(&handle).unwrap());
    handle.spawn(events);
    
    let m = Timeout::new(Duration::new(1, 0), &handle).unwrap()
        .and_then(|_| {
            println!("Tick");
            Timeout::new(Duration::new(1, 0), &handle).unwrap()
        })
        .and_then(|_| {
            println!("Tick");
            Timeout::new(Duration::new(1, 0), &handle).unwrap()
        })
        .and_then(|_| {
            println!("Tick");
            Timeout::new(Duration::new(1, 0), &handle).unwrap()
        })
        .map(|_| {
            println!("Boom");
        });

    core.run(m).unwrap();
}

