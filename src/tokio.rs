
// Async stuff

use {Device, Event};
use mio::{self, Evented};
use mio::unix::EventedFd;
use tokio_core::reactor::{PollEvented, Handle};
use futures::{Poll, Future, Async};

use std::io::{self, ErrorKind};
use std::mem::replace;
use std::os::unix::io::AsRawFd;

#[cfg(feature = "tokio")]
impl Evented for Device {
    fn register(&self,
                poll: &mio::Poll,
                token: mio::Token,
                interest: mio::Ready,
                opts: mio::PollOpt)
                -> io::Result<()>
    {
        poll.register(&EventedFd(&self.as_raw_fd()), token, interest, opts)
    }
    fn reregister(&self,
                  poll: &mio::Poll,
                  token: mio::Token,
                  interest: mio::Ready,
                  opts: mio::PollOpt)
                  -> io::Result<()>
    {
        poll.reregister(&EventedFd(&self.as_raw_fd()), token, interest, opts)   
    }
    fn deregister(&self, poll: &mio::Poll) -> io::Result<()>
    {
        poll.deregister(&EventedFd(&self.as_raw_fd()))
    }
}

/// Async version
// pub struct Device {
    
// }

pub enum EventFuture {
    Waiting { 
        io: PollEvented<Device>,
    },
    Empty,
}

impl EventFuture {
    pub fn new(dev: Device, handle: &Handle) -> io::Result<EventFuture> {
        let mut dev = dev;
        dev.set_nonblocking(true)?;
        Ok(EventFuture::Waiting { io: PollEvented::new(dev,handle)?  })
    }
    
    pub fn get_mut(&mut self) -> &mut Device {
        match *self {
            EventFuture::Waiting { ref mut io } => io.get_mut(),
            _ => panic!("")
        }
    }
}

#[cfg(feature = "tokio")]
impl Future for EventFuture {
    type Item = (EventFuture, Event);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(EventFuture, Event), io::Error> {
        let ev = match *self {
            EventFuture::Waiting { ref mut io } => {
                if let Async::NotReady = io.poll_read() {
                    return Ok(Async::NotReady)
                }
                match io.get_mut().read_event() {
                    Ok(ev) => ev,
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                        io.need_read();
                        return Ok(Async::NotReady);
                    }
                    Err(err) => return Err(err)
                }
            }
            EventFuture::Empty => panic!("Polled on EventFuture after it's done"),
        };

        match replace(self, EventFuture::Empty) {
            dev@EventFuture::Waiting { .. } => Ok((dev, ev).into()),
            _ => panic!("")
        }
    }
}
