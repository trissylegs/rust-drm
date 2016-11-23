

// Dev notes:
//
// * String lengths do not include null terminators.  Therefore
// strings returned by drm drivers do not need to be null terminated.
// This is not much of an issue in rust. Generally str::from_utf8
// works.  I'm assuming drivers will always return ascii
// strings. (However we'll still check it's well formed utf8)

extern crate libc;
#[macro_use]
extern crate bitflags;
extern crate memmap;

mod ioctl_vals;
mod ffi;
mod fourcc;
pub mod mode;

use mode::Id;
use ioctl_vals::*;
use libc::ioctl;
use std::ops::{Deref, DerefMut};
use std::fs::{self, File, OpenOptions};
use std::io::BufReader;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::io;
use std::iter::repeat;
use std::mem::{size_of, transmute};
use std::os::raw::*;
use std::os::unix::prelude::*;
use std::path::{Path, PathBuf};
use std::str;
use std::string::FromUtf8Error;
use std::time::Instant;

#[allow(dead_code)]
mod consts {
    pub const DRM_EVENT_VBLANK: u32 = 0x01;
    pub const DRM_EVENT_FLIP_COMPLETE: u32 = 0x02;
    pub const DRM_VBLANK_ABSOLUTE: u32 = 0;
    pub const DRM_VBLANK_RELATIVE: u32 = 1;
    pub const DRM_VBLANK_HIGH_CRTC_MASK: u32 = 62;
    pub const DRM_VBLANK_EVENT: u32 = 67108864;
    pub const DRM_VBLANK_FLIP: u32 = 134217728;
    pub const DRM_VBLANK_NEXTONMISS: u32 = 268435456;
    pub const DRM_VBLANK_SECONDARY: u32 = 536870912;
    pub const DRM_VBLANK_SIGNAL: u32 = 1073741824;
    pub const DRM_MODE_CURSOR_BO: u32 = 0x01;
    pub const DRM_MODE_CURSOR_MOVE: u32 = 0x02;
}    

fn check_ioctl_err(ret: c_int) -> io::Result<()> {
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

const BUFFER_CAPACITY: usize = 1024;

/// Open fd for a dri device. Such as /dev/dri/card0.
#[derive(Debug)]
pub struct Device {
    fd: BufReader<File>,
}

impl Device {
    /// List the cards found at the "Usual place" (/dev/dri).
    ///
    /// This is mostly for convienience. It's simply iterating over: /dev/dri/card*
    ///
    /// A more nuanced solution might need to be implemented.
    /// Such as support control and render nodes.
    pub fn cards() -> io::Result<Box<Iterator<Item=PathBuf>>> {
        // Read this hardcoded directory. (Normal place for Linux)
        Ok(Box::new({
            try!(fs::read_dir("/dev/dri"))
            // Filter out io erros
                .filter_map(Result::ok)
            // Extract the pathname
                .map(|e| e.path())
            // Filter out files that aren't cards
                .filter(|p| p.file_name()
                        .and_then(|f| f.to_str())
                        .map(|f| f.starts_with("card"))
                        .unwrap_or(false))
        }) as Box<_>)
    }

    /// Opens a card at a particular path.
    pub fn open<P>(path: P) -> io::Result<Device>
        where P: AsRef<Path>,
    {
        // TODO: Check if it's a valid card. (By major/minor number)
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map(|f| {
                Device { fd: BufReader::with_capacity(BUFFER_CAPACITY, f) }
            })
    }

    /// Grab the first card availble (by filename order), this is
    /// probably not what you want to do. Except when:
    /// 
    /// * You have a stupid demo application (like the ones in ../examples)
    /// * Fallback for a configurable program that is not configured.
    /// * You have a completely bespoke system and you know which card should be there.
    ///   (eg embedded systems)
    ///
    /// (i.e don't be asinine and force future users to use hacks to
    /// make your program work on multicard systems)
    pub fn first_card() -> io::Result<Device> {
        let mut cards: Vec<_> = try!(Device::cards()).collect();
        cards.sort();
        match cards.first() {
            None => Err(io::Error::new(io::ErrorKind::NotFound,
                                       "No cards found in /dev/dri".to_string())),
            Some(p) => Device::open(p)
        }
    }

    fn ioctl<T:DrmIoctl>(&self, arg: &mut T) -> io::Result<()> {
        loop {
            let ret = unsafe {
                ioctl(self.as_raw_fd(), T::request(), arg.as_ptr())
            };
            return match check_ioctl_err(ret) {
                Ok(ok) => Ok(ok),
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => Err(e)
            }
        }
    }

    /// Set this process to be the current card master.
    ///
    /// Returns a Guard that will unset master on drop.  The card
    /// master can set the mode of the device and can update the frame
    /// buffer.
    ///
    /// Returns a Invalid Arguement error (`ErrorKind::InvalidInput`)
    /// when another process already is already card master.
    pub fn set_master<'a>(&'a mut self) -> io::Result<Master<'a>> {
        let ret = unsafe {
            ioctl(self.as_raw_fd(), DRM_IOCTL_SET_MASTER, 0)
        };
        match check_ioctl_err(ret) {
            Ok(_) => Ok(Master { dev: self }),
            Err(err) => Err(err),
        }
    }

    /// Get driver version information.
    pub fn version(&self) -> io::Result<Version> {
        let mut version: ffi::version = Default::default();

        try!(self.ioctl(&mut version));

        let mut name: Vec<u8> = repeat(0).take(version.name_len).collect();
        version.name = name.as_mut_ptr() as *mut i8;

        let mut date: Vec<u8> = repeat(0).take(version.date_len).collect();
        version.date = date.as_mut_ptr() as *mut i8;

        let mut desc: Vec<u8> = repeat(0).take(version.desc_len).collect();
        version.desc = desc.as_mut_ptr() as *mut i8;

        try!(self.ioctl(&mut version));
        Ok(Version {
            major: version.version_major,
            minor: version.version_minor,
            patchlevel: version.version_patchlevel,
            name: String::from_utf8(name).map_err(FromUtf8Error::into_bytes),
            date: String::from_utf8(date).map_err(FromUtf8Error::into_bytes),
            desc: String::from_utf8(desc).map_err(FromUtf8Error::into_bytes),
        })
    }

    pub fn get_resources(&self) -> io::Result<mode::Resources> {
        mode::Resources::get(self)
    }

    pub fn get<T: mode::Resource>(&self, id: Id<T>) -> io::Result<T> {
        T::get(self, id)
    }

    /// Fetches the busid of the card.
    ///
    /// Dev notes: on my system this is an empty string. In `xf86drm.h`
    /// there are functions that use this string to open the device.
    /// I assume it might be realted to  path names in `/sys/`.
    pub fn busid(&self) -> io::Result<String> {
        let mut unique = ffi::unique::default();
        impl DrmIoctl for ffi::unique {
            fn request() -> c_ulong { DRM_IOCTL_GET_UNIQUE }
        }

        try!(self.ioctl(&mut unique));
        if unique.unique_len > 0 {
            let mut s: Vec<u8> = repeat(0).collect();
            
            unique.unique = s.as_mut_ptr() as *mut c_char;
            try!(self.ioctl(&mut unique));

            String::from_utf8(s)
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
        } else {
            Ok(String::new())
        }
    }

    /// Request a vblank event, that can be late read by an read_event.
    ///
    /// `crtc_num` is the index of the CRTC id returned by `Resources::crtcs`.
    pub fn request_vblank(&self, data: usize, crtc_num: u32) -> io::Result<()> {

        // Originally there were at most 2 CRTC's and so a "Secondary"
        // flag was sufficient to switch between them.
        // Now we have more that 2 so things are weird.
        let pipe = crtc_num;
        let mut seq_type = if pipe > 1 {
            ((pipe << 1) as u32) & consts::DRM_VBLANK_HIGH_CRTC_MASK
        } else if pipe > 0 {
            consts::DRM_VBLANK_SECONDARY
        } else {
            0
        };

        seq_type |= consts::DRM_VBLANK_RELATIVE;
        seq_type |= consts::DRM_VBLANK_EVENT;

        impl DrmIoctl for ffi::wait_vblank_request {
            fn request() -> c_ulong { DRM_IOCTL_WAIT_VBLANK }
        }
        let mut req = ffi::wait_vblank_request {
            type_: unsafe { transmute(seq_type) },
            sequence: 1,
            signal: data as c_ulong,
        };
        
        self.ioctl(&mut req)
    }

    /// Reads the next available event.
    ///
    /// # Errors
    ///
    /// * If no events are availble an error with kind
    ///   `ErrorKind::WouldBlock` is returned.
    /// * If a syscall is interrupted `ErrorKind::Interrupted`
    /// * If suprising data is returned `ErrorKind::InvalidData`.
    /// * Any other `io:Error` from the kernel.
    pub fn read_event(&mut self) -> io::Result<Event> {

        // This needs a good cleanup.
        
        let nbytes;
        let result = {
            let buf: &[u8] = try!(self.fd.fill_buf());

            if buf.len() == 0 {
                // I have a feeling this won't happen.
                // But if it does this is most sensible.
                return Err(io::Error::new(io::ErrorKind::WouldBlock, "No events available"));
            }

            assert!(buf.len() >= size_of::<ffi::event>(),
                    "Malformed event from drm device: {} < {}",
                    buf.len(), size_of::<ffi::event>());

            // Doing things the C way. There shouldn't be any byte-order issues.
            let ev_header: ffi::event = unsafe {
                *transmute::<_, *const ffi::event>(buf.as_ptr())
            };

            if buf.len() < ev_header.length as usize {
                nbytes = buf.len();
                Err(io::Error::new(ErrorKind::InvalidData, "Short DRM event"))
            } else {
                nbytes = ev_header.length as usize;
                match ev_header.type_ {
                    consts::DRM_EVENT_VBLANK | consts::DRM_EVENT_FLIP_COMPLETE => {
                        let ev: ffi::event_vblank = unsafe {
                            *transmute::<_, *const ffi::event_vblank>(buf.as_ptr())
                        };
                        
                        // This is not good.  This works fine on linux
                        // for now, as Instant is stored as a timespec.
                        // But it may not be portable and it may break.
                        let at: Instant = unsafe {
                            transmute(libc::timespec { tv_sec: ev.tv_sec as i64,
                                                       tv_nsec: ev.tv_usec as i64 * 1000 })
                        };
                        
                        match ev.base.type_ {
                            consts::DRM_EVENT_VBLANK => Ok(Event::VBlank {
                                seq: ev.sequence,
                                tv: at,
                                user: ev.user_data }),
                            consts::DRM_EVENT_FLIP_COMPLETE => Ok(Event::PageFlip {
                                seq: ev.sequence,
                                tv: at,
                                user: ev.user_data }),
                            _ => unreachable!(),
                        }
                    }
                    _ => Ok(Event::Unknown),
                }
            }
        };
        self.fd.consume(nbytes);
        result
    }

    /// Get the capability or value associated with a given capability.
    pub fn capability(&self, cap: Capability) -> io::Result<u64> {
        let mut call = ffi::get_cap {
            capability: cap as u64,
            value: 0,
        };
        impl DrmIoctl for ffi::get_cap {
            fn request() -> c_ulong { DRM_IOCTL_GET_CAP }
        }

        try!(self.ioctl(&mut call));
        Ok(call.value)
    }
}

#[repr(u64)]
pub enum Capability {
    DumbBuffer = 0x1,
    VblankHighCrtc = 0x2,
    DumbPreferredDepth = 0x3,
    DumbPreferShadow = 0x4,
    Prime = 0x5,
    // I think these a bit masked results?
    // PrimeImport = 0x1,
    // PrimeExport = 0x2,
    TimestampMonotonic = 0x6,
    AsyncPageFlip = 0x7,
    CursorWidth = 0x8,
    CursorHeight = 0x9,
    Addfb2Modifiers = 0x10,
}
 
impl AsRawFd for Device {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.get_ref().as_raw_fd()
    }
}
impl IntoRawFd for Device {
    fn into_raw_fd(self) -> RawFd {
        self.fd.into_inner().into_raw_fd()
    }
}
impl FromRawFd for Device {
    unsafe fn from_raw_fd(fd: RawFd) -> Device {
        Device { fd: BufReader::with_capacity(BUFFER_CAPACITY, File::from_raw_fd(fd)) }
    }
}

trait DrmIoctl {
    fn request() -> c_ulong;
    fn as_ptr(&mut self) -> *mut Self {
        self as *mut Self
    }
}

impl DrmIoctl for ffi::version {
    fn request() -> c_ulong { DRM_IOCTL_VERSION }
}

impl DrmIoctl for ffi::mode_cursor {
    fn request() -> c_ulong { DRM_IOCTL_MODE_CURSOR }
}

pub trait GemHandle {
    fn bo_handle(&self) -> u32;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

/// Represents a card when the process is the card master.
///
/// Only one process can control modesetting of a card. To do this
/// they use a `DRM_IOCTL_SET_MASTER`. If it successful modesetting,
/// calls can be performed until `DRM_IOCTL_UNSET_MASTER`, is
/// called or the process exits.
///
/// This object is created when `Device::set_master` is called and
/// when dropped, it will make the Unset call. 
///
/// Due to some mutability constraits this also implements `DerefMut`
/// for the associated card.
/// (Essentailly it used to be impossible to read events and have card master
/// at the same time, because `Read` requires mutable access).
#[derive(Debug)]
pub struct Master<'a> {
    dev: &'a mut Device,
}

impl<'a> Deref for Master<'a> {
    type Target = Device;
    fn deref(&self) -> &Device {
        self.dev
    }
}
impl<'a> DerefMut for Master<'a> {
    fn deref_mut(&mut self) -> &mut Device {
        self.dev
    }
}

impl<'a> Drop for Master<'a> {
    fn drop(&mut self) {
        let _ret = unsafe {
            ioctl(self.dev.as_raw_fd(), DRM_IOCTL_DROP_MASTER, 0)
        };
    }
}

impl<'a> Master<'a>
{
    pub fn set_cursor<B>(&self, crtc_id: Id<mode::Crtc>, bo: &B) -> io::Result<()>
        where B: GemHandle
    {
        let mut arg = ffi::mode_cursor {
            flags: consts::DRM_MODE_CURSOR_BO,
            crtc_id: crtc_id.as_u32(),
            width: bo.width(),
            height: bo.height(),
            handle: bo.bo_handle(),
            ..Default::default()
        };

        self.ioctl(&mut arg)
    }

    pub fn move_cursor(&self, crtc_id: Id<mode::Crtc>, x: i32, y: i32) -> io::Result<()> {
        let mut arg = ffi::mode_cursor {
            flags: consts::DRM_MODE_CURSOR_MOVE,
            crtc_id: crtc_id.as_u32(),
            x: x,
            y: y,
            ..Default::default()
        };
        self.ioctl(&mut arg)
    }
    
    /// Set the mode, and the frame buffer.
    ///
    /// The driver will pick a path from the given crtc to the given connectors.
    pub fn set_crtc(&self,
                    crtc_id: Id<mode::Crtc>,
                    fb: Option<Id<mode::Fb>>,
                    x: u32, y: u32,
                    connectors: &[Id<mode::Connector>],
                    mode: Option<&mode::ModeInfo>)
                    -> io::Result<()>
    {
        #[repr(C)]
        struct SetCrtc {
            set_connectors_ptr: u64,
            count_connectors: u32,
            crtc_id: u32,
            fb_id: u32,
            x: u32,
            y: u32,
            gamma_size: u32,
            mode_valid: u32,
	    mode: mode::ModeInfo,
        };
        impl DrmIoctl for SetCrtc {
            fn request() -> c_ulong { DRM_IOCTL_MODE_SETCRTC }
        }
        let mut crtc = SetCrtc {
            set_connectors_ptr: unsafe { transmute(connectors.as_ptr()) },
            count_connectors: connectors.len() as u32,
            crtc_id: crtc_id.as_u32(),
            fb_id: fb.map(|id| id.as_u32()).unwrap_or(0),
            x: x,
            y: y,
            gamma_size: 0,
            mode_valid: if mode.is_some() { 1 } else { 0 },
            mode: match mode {
                None => unsafe { std::mem::zeroed() },
                Some(mode) => { *mode }
            }
        };
        
        self.dev.ioctl(&mut crtc)
    }
}

/// Version information.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Version {
    major: i32,
    minor: i32,
    patchlevel: i32,
    name: Result<String, Vec<u8>>,
    date: Result<String, Vec<u8>>,
    desc: Result<String, Vec<u8>>
}


impl Version {
    /// Major, minor and patch number of the driver software.
    pub fn number(&self) -> (u32, u32, u32) {
        (self.major as u32, self.minor as u32, self.patchlevel as u32)
    }
    /// Name of the driver software.
    pub fn name(&self) -> Result<&str, &[u8]> {
        self.name.as_ref().map(String::as_str).map_err(Vec::as_slice)
    }
    /// Date published of the driver software. In format: YYYYMMDD
    pub fn date(&self) -> Result<&str, &[u8]> {
        self.date.as_ref().map(String::as_str).map_err(Vec::as_slice)
    }
    /// A description of the driver.
    pub fn desc(&self) -> Result<&str, &[u8]> {
        self.desc.as_ref().map(String::as_str).map_err(Vec::as_slice)
    }
}


/// Type of a drm device event.
#[derive(Copy,Clone,Debug,Eq,PartialEq,Hash)]
pub enum EventType {
    /// Event is start of vblank.
    VBlank,
    /// Event is after page flip has finshed.
    FlipComplete,
}

/// Event types read from device. Event's are sent only after a
/// request vblank has been called or a page flip has been issued.
///
/// Warning: `Instant` values may be from the future.
/// The driver appears to return when a VBlank *will* happen. Not when it did happen.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Event {
    /// Event type unknown to this library.
    Unknown,
    VBlank { seq: u32, tv: Instant, user: u64 },
    PageFlip { seq: u32, tv: Instant, user: u64 },
}

// First attempt at event.
// I should probably remove it.
// But it is actaully cleaner than the current one.

/// Device event.
///
/// Event's are a way for the compositor to tell us when vblank or a
/// pageflip occurs. They are accessed via the read syscall and the
/// device can be added to select, poll or epoll to check if has any
/// event's available.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Event_ {
    type_: EventType,
    user_data: u64,
    time: Instant,
    sequence: u32,
}

#[allow(dead_code)]
impl Event_
{
    /// User data passed to event request or page flip command.
    /// (vblank requests can only accept usize, but pageflip commands
    /// can pass u64, this is probably due to the relative ages of apis)
    pub fn user_data(&self) -> u64 { self.user_data }
    /// Type of the event. Either a `VBlank` or `FlipComplete`.
    pub fn event_type(&self) -> EventType { self.type_ }
    /// Monotonic sequence number generated by the driver.
    pub fn sequence(&self) -> u32 { self.sequence }
    /// The time of the event.
    /// 
    /// This time may be in the future. So avoid calling
    /// `.elapsed`. Consider only comparing it to other times produced
    /// by events.
    pub fn time_val(&self) -> Instant { self.time }
    
    /// Read the next event on the socket.
    ///
    /// This reads a single event from the socket returns it.
    /// This requires mut because:
    ///
    /// * Consuming an event could cause problems if done in multiple places
    ///   as mut guarentees on aliasing, we can safely read it.
    /// * &mut is required for calling Read::read.
    fn read_event(dev: &mut Device) -> io::Result<Event_>
    {
        // There are several transmutes here.
        // transumte does require both types have the same 
        
        // This is kinda gross, but it saves me from manually
        // implementing an iterator (TODO!).
        const EVENT_BYTES: usize = 32;
        let mut buf = [0; EVENT_BYTES];
        
        let n = try!(dev.fd.read(&mut buf[..]));
        if n == 0 {
            return Err(io::Error::new(io::ErrorKind::WouldBlock, "no events available"));
        }
        // TODO: do something smarter than panicing.
        // (Future versions of drm might add smaller events).
        // (We don't get partial events. DRM guarantess that).
        assert_eq!(EVENT_BYTES, n);

        let ev: ffi::event_vblank = unsafe {
            transmute(buf)
        };

        let type_ = match ev.base.type_ {
            consts::DRM_EVENT_VBLANK => EventType::VBlank,
            consts::DRM_EVENT_FLIP_COMPLETE => EventType::FlipComplete,
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData,
                                           "Unknow DRM event type")),
        };

        // Well... std::time::Instant is nice.
        // There's no nice way to create one.
        let at: Instant = unsafe {
            transmute(libc::timespec { tv_sec: ev.tv_sec as i64,
                                       tv_nsec: ev.tv_usec as i64 * 1000 })
        };

        println!("current instant: {:?}", Instant::now());
        println!("   evil instant: {:?}", at);

        // // Calling this will check if at is in the past.
        // // Otherwise it will panic.
        // at.elapsed();
        
        // Turns out that always panics: my driver returns vblank
        // events from the Future.  I'm assuming that the driver is
        // returned the time vblank actually occurs. But the event is
        // sent before it occurs. 

        Ok(Event_ {
            type_: type_,
            user_data: ev.user_data,
            time: at,
            sequence: ev.sequence,
        })
    }
}
