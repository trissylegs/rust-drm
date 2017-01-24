
#![feature(conservative_impl_trait)]

extern crate drm;

#[cfg(feature="tokio")] extern crate tokio_core;
#[cfg(feature="tokio")] extern crate futures;


#[cfg(feature="tokio")] use tokio_core::reactor::Handle;
#[cfg(feature="tokio")] use tokio_core::reactor::Timeout;
#[cfg(feature="tokio")] use futures::Future;
#[cfg(feature="tokio")] use drm::tokio::EventFuture;
#[cfg(feature="tokio")] use std::time::Duration;

#[cfg(feature="tokio")] 
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

#[cfg(feature="tokio")] 
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

#[cfg(not(feature="tokio"))]
fn main() {
    panic!("tokio example requires tokio feature");
}
