
mod ffi;

use std::{io, fmt, str};
use std::mem::{transmute, zeroed};
use std::os::raw::c_ulong;
use std::cmp::Ordering;
use super::ioctl_vals::*;
use super::Device;
use super::fourcc::FourCC;
use super::DrmIoctl;
use std::marker::PhantomData;
use memmap::{Mmap, Protection};
use std::hash::{Hash, Hasher};

/// Id is a 32-bit integer that represents an object in the driver.
///
/// This information can change at any time, due to physical hardware
/// changes, driver behavior, other programs, etc. So the Id can be used
/// to get the information again using `Device::get`.
///
/// Most API's use Id's for passing information back to the kernel as well.
///
/// The phantom type parameter allows can unsure that you don't
/// accidently mix id's, and it makes Device::get very convienient.
///
/// let thing = device.get(id).unwrap();
pub struct Id<T>(u32, PhantomData<*const T>);

// derive Copy requites that T is Copy. As Id's don't actually hold
// any non-copy data we can declare it manually.
impl<T> Copy for Id<T> {}
impl<T> Clone for Id<T> {
    #[inline]
    fn clone(&self) -> Id<T> {
        Id(self.0, PhantomData)
    }
}

impl<T> PartialEq for Id<T> {
    #[inline]
    fn eq(&self, &rhs: &Id<T>) -> bool { self.0 == rhs.0 }
}
impl<T> Eq for Id<T> {}

impl<T> Ord for Id<T> {
    #[inline]
    fn cmp(&self, rhs: &Id<T>) -> Ordering {
        self.0.cmp(&rhs.0)
    }
}
impl<T> PartialOrd for Id<T> {
    #[inline]
    fn partial_cmp(&self, rhs: &Id<T>) -> Option<Ordering> {
        Some(self.cmp(&rhs))
    }
}

impl<T> Hash for Id<T> {
    #[inline]
    fn hash<H>(&self, state: &mut H) where H: Hasher
    {
        self.0.hash(state)
    }
}
impl<T: Resource> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id({})", self.0)
    }
}

impl<T: Resource> Id<T> {
    /// Create an Id wrapper of a u32. As the id maybe not actually
    /// exist or be the correct type, so this method is unsafe.
    pub unsafe fn from_u32(id: u32) -> Option<Id<T>> {
        if id == 0 { None } else { Some(Id(id, PhantomData)) }
    }
    /// Get the value of the id.
    pub        fn as_u32(&self) -> u32 { self.0 }
}

pub trait Resource: Sized {
    fn get(dev: &Device, id: Id<Self>) -> io::Result<Self>;
}

// /// KMS ioctl's accept 64 bit integers for pointers. So we must
// /// convert between pointers and u64.
// fn from_u64<T>(u: u64) -> *mut T {
//     unsafe {
//         assert!(u <= ::std::usize::MAX as u64);
//         ::std::mem::transmute(u as usize)
//     }
// }

/// KMS ioctl's accept 64 bit integers for pointers. So we must
/// convert between pointers and u64.
fn from_ptr<T>(p: *mut T) -> u64 {
    unsafe {
        transmute::<_, usize>(p) as u64
    }
}


/// Resource lists all the Resources avaible on the card.
#[derive(Debug, Clone)]
pub struct Resources
{
    fbs: Vec<Id<Fb>>,
    crtcs: Vec<Id<Crtc>>,
    connectors: Vec<Id<Connector>>,
    encoders: Vec<Id<Encoder>>,
    min_size: (u32, u32),
    max_size: (u32, u32),
}

impl DrmIoctl for ffi::card_res
{
    fn request() -> c_ulong { DRM_IOCTL_MODE_GETRESOURCES }
}

impl Resources
{
    pub fn get(dev: &Device) -> io::Result<Resources>
    {
        let mut fbs = Vec::new();
        let mut crtcs = Vec::new();
        let mut connectors = Vec::new();
        let mut encoders = Vec::new();
        loop {
            let mut get_res = ffi::card_res::default();
            try!(dev.ioctl(&mut get_res));

            let counts = get_res.clone();

            macro_rules! alloc_ids {
                ($vec:ident, $count:expr, $ptr:expr) => {
                    if $count > 0 {
                        $vec.resize($count as usize, Id(0, PhantomData));
                        $ptr = from_ptr($vec.as_mut_ptr());
                    }
                }
            }

            alloc_ids!(fbs,        get_res.count_fbs,        get_res.fb_id_ptr);
            alloc_ids!(crtcs,      get_res.count_crtcs,      get_res.crtc_id_ptr);
            alloc_ids!(connectors, get_res.count_connectors, get_res.connector_id_ptr);
            alloc_ids!(encoders,   get_res.count_encoders,   get_res.encoder_id_ptr);

            try!(dev.ioctl(&mut get_res));
            
            if get_res.count_fbs != counts.count_fbs || 
                get_res.count_crtcs != counts.count_crtcs || 
                get_res.count_connectors != counts.count_connectors || 
                get_res.count_encoders != counts.count_encoders
            {
                continue;
            }
            return Ok(Resources {
                fbs: fbs,
                crtcs: crtcs,
                connectors: connectors,
                encoders: encoders,
                min_size: (get_res.min_width, get_res.min_height),
                max_size: (get_res.max_width, get_res.max_height),
            });
        }
    }

    pub fn fbs(&self) -> &[Id<Fb>]
    {
        self.fbs.as_slice()
    }
    pub fn crtcs(&self) -> &[Id<Crtc>]
    {
        self.crtcs.as_slice()
    }
    pub fn connectors(&self) -> &[Id<Connector>]
    {
        self.connectors.as_slice()
    }
    pub fn encoders(&self) -> &[Id<Encoder>]
    {
        self.encoders.as_slice()
    }

    /// The minimum resolution supported by this card.
    pub fn min_size(&self) -> (u32, u32) { self.min_size }
    /// The maximum resolution supported by this card.
    pub fn max_size(&self) -> (u32, u32) { self.max_size }
    
}

/// A connector represents the final output.
///
/// Usually a physical display connected by a cable.
#[derive(Debug, Clone)]
pub struct Connector
{
    connector_id: Id<Connector>,
    encoder_id: Option<Id<Encoder>>,
    connector_type: ConnectorType,
    connector_type_id: u32,
    connection: Connection,
    mm_size: (u32, u32),
    subpixel: Subpixel,
    modes: Vec<ModeInfo>,
    props: Vec<Id<Property>>,
    prop_values: Vec<i64>,
    encoders: Vec<Id<Encoder>>,
}

impl DrmIoctl for ffi::get_connector
{
    fn request() -> c_ulong { DRM_IOCTL_MODE_GETCONNECTOR }
}

impl Resource for Connector
{
    fn get(dev: &Device, id: Id<Connector>) -> io::Result<Connector>
    {
        unsafe {
            let mut modes = Vec::new();
            let mut props = Vec::new();
            let mut prop_values = Vec::new();
            let mut encoders = Vec::new();
            loop {
                let mut conn = ffi::get_connector::default();
                conn.connector_id = id.0;

                try!(dev.ioctl(&mut conn));
                let counts = conn;

                modes.resize(conn.count_modes as usize,
                             ModeInfo::from(ffi::modeinfo::default()));
                conn.modes_ptr = from_ptr(modes.as_mut_ptr() as *mut ffi::modeinfo);

                props.resize(conn.count_props as usize, Id(0, PhantomData));                
                conn.props_ptr = from_ptr(props.as_mut_ptr() as *mut u32);

                prop_values.resize(conn.count_props as usize, 0);
                conn.prop_values_ptr = from_ptr(prop_values.as_mut_ptr() as *mut u32);

                encoders.resize(conn.count_encoders as usize, Id(0, PhantomData));
                conn.encoders_ptr = from_ptr(encoders.as_mut_ptr() as *mut u32);
                
                try!(dev.ioctl(&mut conn));
                if counts.count_props < conn.count_props ||
                    counts.count_modes < conn.count_modes ||
                    counts.count_encoders < conn.count_encoders
                {
                    continue;
                }
                return Ok(Connector {
                    connector_id: id,
                    encoder_id: Id::from_u32(conn.encoder_id),
                    connector_type: ConnectorType::from_u32(conn.connector_type),
                    connector_type_id: conn.connector_type_id,
                    connection: Connection::from_u32(conn.connection),
                    mm_size: (conn.mm_width, conn.mm_height),
                    subpixel: Subpixel::from_u32(conn.subpixel),
                    modes: modes,
                    props: props,
                    prop_values: prop_values,
                    encoders: encoders,
                });
            }
        }
    }
}


impl Connector
{
    /// Get an iterator over the properties of this connector and the
    /// values they have been set to.
    pub fn props<'a>(&'a self) -> Props<'a>
    {
        Props {
            props: self.props.as_slice(),
            prop_values: self.prop_values.as_slice()
        }
    }

    /// List of avaible modes on this connector.
    pub fn modes(&self) -> &[ModeInfo]
    {
        self.modes.as_ref()
    }

    /// List of encoders that can be used with this connector.
    pub fn possible_encoders(&self) -> &[Id<Encoder>]
    {
        self.encoders.as_ref()
    }

    /// Show the current encoder being used.
    pub fn encoder_id(&self) -> Option<Id<Encoder>>
    {
        self.encoder_id
    }

    /// Get the id of this encoder.
    pub fn connector_id(&self) -> Id<Connector>
    {
        self.connector_id
    }

    /// Get the type of connector.
    pub fn connector_type(&self) -> ConnectorType
    {
        self.connector_type
    }

    /// Get the state of the connector. (Connected/Disconnected).
    pub fn connection(&self) -> Connection
    {
        self.connection
    }

    /// Get the subpixel layout. (The positions the colors are layed out on the screen).
    pub fn subpixel(&self) -> Subpixel
    {
        self.subpixel
    }
}

/// Types of connectors.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum ConnectorType
{
    Unknown = 0,
    VGA = 1,
    DVII = 2,
    DVID = 3,
    DVIA = 4,
    Composite = 5,
    SVIDEO = 6,
    LVDS = 7,
    Component = 8,
    _9PinDIN = 9,
    DisplayPort = 10,
    HDMIA = 11,
    HDMIB = 12,
    TV = 13,
    /// Embedded display port eDP.
    EDP = 14,
    VIRTUAL = 15,
    DSI = 16,
}
impl ConnectorType
{
    fn from_u32(u: u32) -> ConnectorType
    {
        match u {
            1 => ConnectorType::VGA,
            2 => ConnectorType::DVII,
            3 => ConnectorType::DVID,
            4 => ConnectorType::DVIA,
            5 => ConnectorType::Composite,
            6 => ConnectorType::SVIDEO,
            7 => ConnectorType::LVDS,
            8 => ConnectorType::Component,
            9 => ConnectorType::_9PinDIN,
            10 => ConnectorType::DisplayPort,
            11 => ConnectorType::HDMIA,
            12 => ConnectorType::HDMIB,
            13 => ConnectorType::TV,
            14 => ConnectorType::EDP,
            15 => ConnectorType::VIRTUAL,
            16 => ConnectorType::DSI,
            _ => ConnectorType::Unknown,
        }            
    }
}
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Connection
{
    Connected = 1,
    Disconnected = 2,
    Unknown = 3,
}
impl Connection
{
    fn from_u32(u: u32) -> Connection
    {
        match u {
            1 => Connection::Connected,
            2 => Connection::Disconnected,
            _ => Connection::Unknown,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Subpixel
{
    Unknown = 1,
    HorizontalRGB = 2,
    HorizontalBGR = 3,
    VerticalRGB   = 4,
    VerticalBGR   = 5,
    None = 6
}
impl Subpixel {
    fn from_u32(u: u32) -> Subpixel
    {
        match u {
            2 => Subpixel::HorizontalRGB,
            3 => Subpixel::HorizontalBGR,
            4 => Subpixel::VerticalRGB  ,
            5 => Subpixel::VerticalBGR  ,
            6 => Subpixel::None,
            _ => Subpixel::Unknown,
        }
    }
}

#[derive(Clone)]
pub struct Props<'a>
{
    props: &'a [Id<Property>],
    prop_values: &'a [i64],
}
impl<'a> Iterator for Props<'a>
{
    type Item = (Id<Property>, i64);
    fn next(&mut self) -> Option<(Id<Property>, i64)>
    {
        if self.props.is_empty() | self.prop_values.is_empty() {
            None
        } else {
            let r = (self.props[0].clone(), self.prop_values[0]);
            self.props = &self.props[1..];
            self.prop_values = &self.prop_values[1..];
            Some(r)
        }
    }
}

impl<'a> fmt::Debug for Props<'a>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result
    {
        fmt.debug_list()
            .entries(self.props.iter().zip(self.prop_values.iter()))
            .finish()
    }
}

/// The mode configuration fo the screen.
///
/// Set's the resolution and refresh rate. Note that screens usually
/// cannot handle arbitary values here.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ModeInfo
{
    pub clock: u32,
    pub hdisplay: u16,
    pub hsync_start: u16,
    pub hsync_end: u16,
    pub htotal: u16,
    pub hskew: u16,
    pub vdisplay: u16,
    pub vsync_start: u16,
    pub vsync_end: u16,
    pub vtotal: u16,
    pub vscan: u16,
    pub vrefresh: u32,
    pub flags: ModeFlags,
    pub type_: ModeType,
    name: [u8; 32usize],
}

impl ModeInfo
{
    /// Get's the name of the current mode.
    pub fn name(&self) -> &str
    {
        let len: usize = (&self.name[..]).iter().position(|b| *b == 0)
            .unwrap_or(32);

        // TODO: handle the case where the kernel gives us non-utf8.
        str::from_utf8(&self.name[..len])
            .expect("mode name not valid utf8")
    }
}

impl From<ffi::modeinfo> for ModeInfo
{
    fn from(mi: ffi::modeinfo) -> ModeInfo
    {
        unsafe { transmute(mi) }
    }
}

impl fmt::Debug for ModeInfo
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result
    {
        fmt.debug_struct("ModeInfo")
            .field("clock", &self.clock)
            .field("hdisplay", &self.hdisplay)
            .field("hsync_start", &self.hsync_start)
            .field("hsync_end", &self.hsync_end)
            .field("htotal", &self.htotal)
            .field("hskew", &self.hskew)
            .field("vdisplay", &self.vdisplay)
            .field("vsync_start", &self.vsync_start)
            .field("vsync_end", &self.vsync_end)
            .field("vtotal", &self.vtotal)
            .field("vscan", &self.vscan)
            .field("vrefresh", &self.vrefresh)
            .field("flags", &self.flags)
            .field("type_", &self.type_)
            .field("name", &self.name())
            .finish()
    }
}


bitflags! {
    pub flags ModeType: u32
    {
        const TYPE_BUILTIN   = (1<<0),
        const TYPE_CLOCK_C   = (1<<1) | TYPE_BUILTIN.bits,
        const TYPE_CRTC_C    = (1<<2) | TYPE_BUILTIN.bits,
        const TYPE_PREFERRED = (1<<3),
        const TYPE_DEFAULT   = (1<<4),
        const TYPE_USERDEF   = (1<<5),
        const TYPE_DRIVER    = (1<<6)
    }
}

bitflags! {
    pub flags ModeFlags: u32
    {
        const FLAG_PHSYNC                    = (1<<0),
        const FLAG_NHSYNC                    = (1<<1),
        const FLAG_PVSYNC                    = (1<<2),
        const FLAG_NVSYNC                    = (1<<3),
        const FLAG_INTERLACE                 = (1<<4),
        const FLAG_DBLSCAN                   = (1<<5),
        const FLAG_CSYNC                     = (1<<6),
        const FLAG_PCSYNC                    = (1<<7),
        const FLAG_NCSYNC                    = (1<<8),
        const FLAG_HSKEW                     = (1<<9),
        const FLAG_BCAST                     = (1<<10),
        const FLAG_PIXMUX                    = (1<<11),
        const FLAG_DBLCLK                    = (1<<12),
        const FLAG_CLKDIV2                   = (1<<13),
        const FLAG_3D_NONE                   = (0<<14),
        const FLAG_3D_FRAME_PACKING          = (1<<14),
        const FLAG_3D_FIELD_ALTERNATIVE      = (2<<14),
        const FLAG_3D_LINE_ALTERNATIVE       = (3<<14),
        const FLAG_3D_SIDE_BY_SIDE_FULL      = (4<<14),
        const FLAG_3D_L_DEPTH                = (5<<14),
        const FLAG_3D_L_DEPTH_GFX_GFX_DEPTH  = (6<<14),
        const FLAG_3D_TOP_AND_BOTTOM         = (7<<14),
        const FLAG_3D_SIDE_BY_SIDE_HALF      = (8<<14),
    }
}

/// A property of a connector.
///
/// Contains extensible information about a connector. For example:
/// EDID information, DPMS, audio configuration, etc.
#[derive(Debug, Clone)]
pub struct Property
{
    prop_id: Id<Property>,
    flags: PropertyFlags,
    name: String,
    values: Vec<i64>,
    // This either contains blobs or enums, not both.
    enums: Vec<PropertyEnum>,
    blob_ids: Vec<u32>,
}

impl DrmIoctl for ffi::get_property
{
    fn request() -> c_ulong { DRM_IOCTL_MODE_GETPROPERTY }
}

impl Resource for Property
{
    fn get(dev: &Device, id: Id<Property>) -> io::Result<Property>
    {
        unsafe {
            let mut prop: ffi::get_property = Default::default();
            prop.prop_id = id.0;

            try!(dev.ioctl(&mut prop));

            let mut values = Vec::new();
            values.resize(prop.count_values as usize, 0);
            prop.values_ptr = from_ptr(values.as_mut_ptr());

            let prop_flags = PropertyFlags::from_bits(prop.flags)
                .expect("Invalid property flag"); // TOOD: Nicer error

            let mut enums = Vec::new();
            let mut blobs = Vec::new();

            if prop.count_enum_blobs > 0 {
                if prop_flags.intersects(PROP_ENUM | PROP_BITMASK) {
                    enums.resize(prop.count_enum_blobs as usize, zeroed());
                    prop.enum_blob_ptr = from_ptr(enums.as_mut_ptr());
                } else if prop_flags.contains(PROP_BLOB) {
                    blobs.resize(prop.count_enum_blobs as usize, 0);
                    prop.enum_blob_ptr = from_ptr(blobs.as_mut_ptr());
                }
            }

            try!(dev.ioctl(&mut prop));

            Ok(Property {
                prop_id: id,
                flags: prop_flags,
                name: {
                    let len = (&prop.name[..]).iter()
                        .position(|&b| b == 0)
                        .unwrap_or(prop.name.len());
                    String::from_utf8_lossy(transmute(&prop.name[..len]))
                        .into_owned()
                },
                values: values,
                enums: enums,
                blob_ids: blobs,
            })
        }
    }
}
impl Property
{
    pub fn flags(&self) -> PropertyFlags
    {
        self.flags
    }
}


bitflags! {
    pub flags PropertyFlags: u32
    {
        const PROP_PENDING      = (1<<0),
        const PROP_RANGE        = (1<<1),
        const PROP_IMMUTABLE    = (1<<2),
        const PROP_ENUM         = (1<<3),
        const PROP_BLOB         = (1<<4),
        const PROP_BITMASK      = (1<<5),
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct PropertyEnum
{
    value: i64,
    name: [u8; 32]
}

impl PropertyEnum
{
    pub fn value(&self) -> i64 { self.value }
    pub fn name(&self) -> &str
    {
        let len = (&self.name[..]).iter().position(|&b| b == 0)
            .unwrap_or(self.name.len());
        str::from_utf8(&self.name[..len])
            .expect("property enum name not utf8")
    }
}

impl fmt::Debug for PropertyEnum
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result
    {
        fmt.debug_struct("PropertyEnum")
            .field("value", &self.value)
            .field("name", &self.name())
            .finish()
    }
}

/// A CRTC the first stage in the encoding pipeline.
///
/// CRTC stands for "Cathode Ray Tube Controller". Which works much
/// like CRTC's did. Display cables behave very similar to how CRTC's
/// recieved input so we still used the same term for the device that
/// controls it.
#[derive(Debug, Clone)]
pub struct Crtc
{
    id: Id<Crtc>,
    x: u32,
    y: u32,
    gamma_size: u32,
    fb_id: Option<Id<Fb>>,
    mode: Option<ModeInfo>,
}

impl DrmIoctl for ffi::crtc
{
    fn request() -> c_ulong { DRM_IOCTL_MODE_GETCRTC }
}

impl Crtc
{
    pub fn id(&self) -> Id<Crtc> { self.id }
    /// Id of the current frame buffer.
    pub fn fb_id(&self) -> Option<Id<Fb>> { self.fb_id }
    /// Information about the current mode.
    pub fn mode(&self) -> Option<&ModeInfo> { self.mode.as_ref() }
    /// Position within the frame buffer being scanned out.
    pub fn pos(&self) -> (u32, u32) { (self.x, self.y) }
}


impl Resource for Crtc
{
    fn get(dev: &Device, id: Id<Crtc>) -> io::Result<Crtc>
    {
        let mut get_crtc = ffi::crtc {
            crtc_id: id.0,
            ..Default::default()
        };
        try!(dev.ioctl(&mut get_crtc));
        Ok(Crtc {
            id: id,
            x: get_crtc.x, y: get_crtc.y, gamma_size: get_crtc.gamma_size,
            fb_id: unsafe { Id::from_u32(get_crtc.fb_id) },
            mode: if get_crtc.mode_valid != 0 {
                Some(ModeInfo::from(get_crtc.mode))
            } else {
                None
            }
        })
    }
}


#[derive(Debug, Clone)]
pub struct Encoder
{
    encoder_id: Id<Encoder>,
    encoder_type: EncoderType,
    crtc_id: Option<Id<Crtc>>,
    // TODO: This is probably the "Pipe" numbers of the CRTC's
    possible_crtcs: u32,
    // TODO: What is this?
    possible_clones: u32,
}

impl DrmIoctl for ffi::get_encoder
{
    fn request() -> c_ulong { DRM_IOCTL_MODE_GETENCODER }
}

impl Resource for Encoder
{
    fn get(dev: &Device, id: Id<Encoder>) -> io::Result<Encoder>
    {
        let mut get_encoder = ffi::get_encoder {
            encoder_id: id.0,
            ..Default::default()
        };
        try!(dev.ioctl(&mut get_encoder));
        
        let crtc_id = unsafe { Id::from_u32(get_encoder.crtc_id) };

        
        Ok(Encoder {
            encoder_id: id,
            encoder_type: EncoderType::from(get_encoder.encoder_type)
                .expect("Unknown encoder type"),
            crtc_id: crtc_id,
            possible_crtcs: get_encoder.possible_crtcs,
            possible_clones: get_encoder.possible_clones,
        })
    }
}

impl Encoder
{
    pub fn id(&self) -> Id<Encoder> { self.encoder_id }
    pub fn crtc_id(&self) -> Option<Id<Crtc>> { self.crtc_id }
    /// The type of encoder.
    pub fn encoder_type(&self) -> EncoderType { self.encoder_type }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum EncoderType
{
    None    = 0,
    Dac     = 1,
    Tmds    = 2,
    Lvds    = 3,
    Tvdac   = 4,
    Virtual = 5,
    Dsi     = 6,
    Dpmst   = 7,
}

impl EncoderType
{
    fn from(u: u32) -> Option<EncoderType>
    {
        match u {
            0 => Some(EncoderType::None),
            1 => Some(EncoderType::Dac),
            2 => Some(EncoderType::Tmds),
            3 => Some(EncoderType::Lvds),
            4 => Some(EncoderType::Tvdac),
            5 => Some(EncoderType::Virtual),
            6 => Some(EncoderType::Dsi),
            7 => Some(EncoderType::Dpmst),
            _ => None,
        }
    }
}

/// A plane is an extra frame buffer that the CRTC's can compose onto
/// the output during scanout.
///
/// Old video game consoles performed rendering by using Sprites and
/// composing them onto the scene using specialised hardware.
///
/// Vidoe devices sitll have this hardware. A common use is for
/// hardware cursors. (DRM has apis for Cursors, it's in the TODO
/// list).
///
/// Intel hardware tends to support 3 non-transparent planes. (So
/// can't be used for cursors). Assuming the planes can be used to
/// compose video content onto the screen. (Possibly PAVP?)
#[derive(Debug, Clone)]
pub struct Plane
{
    plane_id: u32,
    crtc_id: u32,
    fb_id: u32,
    possible_crtcs: u32,
    gamma_size: u32,
    formats: Vec<FourCC>
}

impl DrmIoctl for ffi::get_plane_res
{
    fn request() -> c_ulong { DRM_IOCTL_MODE_GETPLANERESOURCES }
}
impl DrmIoctl for ffi::get_plane
{
    fn request() -> c_ulong { DRM_IOCTL_MODE_GETPLANE }
}
impl Plane {
    /// Planes are not listed in the `Resources` structure so we must fetch a list of them
    /// seperately.
    pub fn get_ids(dev: &Device) -> io::Result<Vec<u32>>
    {
        let mut plane_res = ffi::get_plane_res::default();
        try!(dev.ioctl(&mut plane_res));

        let mut ids = Vec::new();
        ids.resize(plane_res.count_planes as usize, 0);
        plane_res.plane_id_ptr = from_ptr(ids.as_mut_ptr());

        try!(dev.ioctl(&mut plane_res));
        Ok(ids)
    }
    pub fn get(dev: &Device, id: u32) -> io::Result<Plane>
    {
        loop {
            let mut plane = ffi::get_plane::default();
            plane.plane_id = id;

            try!(dev.ioctl(&mut plane));
            let counts = plane;

            let mut formats: Vec<FourCC> = Vec::new();
            if plane.count_format_types > 0 {
                formats.resize(plane.count_format_types as usize, FourCC::Unknown);
                plane.format_type_ptr = from_ptr(formats.as_mut_ptr() as *mut u32);
                try!(dev.ioctl(&mut plane));
                if counts.count_format_types < plane.count_format_types {
                    continue;
                }
            }
            return Ok(Plane {
                plane_id: id,
                crtc_id: plane.crtc_id,
                fb_id: plane.fb_id,
                possible_crtcs: plane.possible_crtcs,
                gamma_size: plane.gamma_size,
                formats: formats
            });
        }
    }
}

/// Blob's are used to get EDID information out of a Property.
///
/// TODO: Make this usable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropertyBlob {
    id: Id<PropertyBlob>,
    data: Vec<u8>,
}

impl DrmIoctl for ffi::get_blob {
    fn request() -> c_ulong {  DRM_IOCTL_MODE_GETPROPBLOB }
}

impl Resource for PropertyBlob
{
    fn get(dev: &Device, blob_id: Id<PropertyBlob>) -> io::Result<PropertyBlob>
    {
        let mut get_blob = ffi::get_blob::default();
        get_blob.blob_id = blob_id.as_u32();

        try!(dev.ioctl(&mut get_blob));
        let mut data = Vec::new();
        if get_blob.length > 0 {
            data.resize(get_blob.length as usize, 0);
        }
        get_blob.data = from_ptr(data.as_mut_ptr());

        try!(dev.ioctl(&mut get_blob));
        Ok(PropertyBlob { id: blob_id, data: data })
    }
}

/// Fb is the DRM representation of frame buffer.
///
/// Either crated by a DRM api, or imported from:
/// 
/// * A GEM handle.
/// * A PRIME fd.
/// * A dma_buf fd.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fb {
    fb_id: Id<Fb>,
    size: (u32, u32),
    pitch: u32,
    bits_per_pixel: u32,
    depth: u32,
    handle: u32
}
impl Resource for Fb {
    fn get(dev: &Device, id: Id<Fb>) -> io::Result<Fb> {
        /// There are two ioctls that make use of drm_mode_fb_cmd.
        /// So we must define DrmIoctl locally.
        #[repr(C)]
        struct GetFb(ffi::fb_cmd);
        impl DrmIoctl for GetFb {
            fn request() -> c_ulong { DRM_IOCTL_MODE_GETFB }
        }
        
        let mut info = GetFb(ffi::fb_cmd::default());
        info.0.fb_id = id.0;
        
        try!(dev.ioctl(&mut info));
        let info = info.0;
        
        Ok(Fb {
            fb_id: id,
            size: (info.width, info.height),
            pitch: info.pitch,
            bits_per_pixel: info.bpp,
            depth: info.depth,
            handle: info.handle,
        })
    }
}

impl Fb
{
    pub fn id(&self) -> Id<Fb> { self.fb_id }

    /// Add a frame buffer using a GEM handle.
    ///
    /// * handle: A gem handle
    /// * (width, height): The width and height in pixels.
    /// * bpp: Bit's per pixel (Eg: 32)
    /// * pitch: Offset between the start of each row in bytes. (ie. width + padding)
    /// * depth: Color depth. (Eg: 24)
    ///
    /// There is an Api for FbAdd2, which allows for mulit-plane
    /// buffers and more useful color format selection.
    pub fn add(dev: &Device, handle: u32,
               width: u32, height: u32,
               bpp: u32, pitch: u32, depth: u32)
               -> io::Result<Fb>
    {
        /// There are two ioctls that make use of drm_mode_fb_cmd.
        /// So we must define DrmIoctl locally.
        #[repr(C)]
        struct AddFb(ffi::fb_cmd);
        impl DrmIoctl for AddFb {
            fn request() -> c_ulong { DRM_IOCTL_MODE_ADDFB }
        }
        
        let mut add_fb = AddFb(ffi::fb_cmd {
            handle: handle,
            width: width, height: height,
            bpp: bpp, pitch: pitch,
            depth: depth,          // TODO: What does this need to be
            ..Default::default()
        });

        dev.ioctl(&mut add_fb)
            .map(|()|
                 Fb {
                     fb_id: unsafe { Id::from_u32(add_fb.0.fb_id).unwrap() },
                     size: (width, height),
                     pitch: pitch,
                     bits_per_pixel: bpp,
                     depth: depth,
                     handle: handle,
                 })
    }

    /// Remove the frame buffer from DRM.
    ///
    /// This will not happen automatically as the `Fb` object is not a
    /// real representation of the Frame buffer, but a struct
    /// containing metadata about it.
    ///
    /// TODO: It might be a good idea to create a real RAII type for Fb.
    pub fn rm(dev: &Device, id: Id<Fb>) -> io::Result<()> {
        struct RmFb(Id<Fb>);
        impl DrmIoctl for RmFb {
            fn request() -> c_ulong { DRM_IOCTL_MODE_RMFB }
        }
        dev.ioctl(&mut RmFb(id))
    }
}

#[allow(dead_code)]
pub struct DumbBuf
{
    handle: u32,
    height: u32,
    width: u32,
    bpp: u32,
    flags: DumbBufFlags,
    pitch: u32,
    size: u64,
    fb: Fb,
    map: Mmap
}

// Todo: destroy DumbBuf
//
// We need access to the underlying Fd to destroy it. Potential solutions:
//  * Put dev.fd behind a Arc. Then we can carry around copies of it.
//  * dup dev.fd, so we can have our own copy of it.
//  * maintain a reference to &Device so we can call Device::ioctl.
//  * Require the use to provide &Device when destroying buffer.
//    then Drop does not destroy the buffer.

impl DumbBuf {
    /// Creates, maps and adds a dumb buf.  I have not checked if it's
    /// wise to do all 3 or let the client deal with it.  But it does
    /// mean we don't have to deal holding onto the device or the
    /// potential error of mapping on the wrong device.
    pub fn create(dev: &Device,
                      width: u32, height: u32,
                      bpp: u32, flags: DumbBufFlags)
                      -> io::Result<DumbBuf>
    {
        // TODO: dumb depth
        const DEPTH: u32 = 24;
        
        let mut create_dumb =
            ffi::create_dumb {
                height: height,
                width: width,
                bpp: bpp,
                flags: flags.bits(),
                ..Default::default()
            };
        impl DrmIoctl for ffi::create_dumb {
            fn request() -> c_ulong { DRM_IOCTL_MODE_CREATE_DUMB }
        }
        try!(dev.ioctl(&mut create_dumb));

        let fb = try!(Fb::add(&dev, create_dumb.handle,
                              width, height, bpp, create_dumb.pitch, DEPTH));
        
        let mut map_dumb =
            ffi::map_dumb {
                handle: create_dumb.handle,
                ..Default::default()
            };
        impl DrmIoctl for ffi::map_dumb {
            fn request() -> c_ulong { DRM_IOCTL_MODE_MAP_DUMB }
        }
        try!(dev.ioctl(&mut map_dumb));
        
        let map = try!(Mmap::open_with_offset(dev.fd.get_ref(),
                                              Protection::ReadWrite,
                                              map_dumb.offset as usize,
                                              create_dumb.size as usize));
        
        Ok(DumbBuf {
            width: width, height: height, bpp: bpp, flags: flags,
            pitch: create_dumb.pitch, size: create_dumb.size,
            handle: create_dumb.handle,
            fb: fb,
            map: map,
        })
    }

    /// Not sure what this used for other than creating/mapping the
    /// frame buffer.
    pub unsafe fn handle(&self) -> u32 { self.handle }

    /// (width, height) in pixels
    pub fn size(&self) -> (u32, u32) { (self.width, self.height) }

    /// Bits per pixel
    pub fn bpp(&self) -> u32 { self.bpp }

    /// Bytes per row
    pub fn pixel(&self) -> u32 { self.pitch }
    
    /// Access the buffer.
    pub fn as_mut(&mut self) -> &mut [u8] {
        unsafe { self.map.as_mut_slice() }
    }

    /// Access the Fb object associated with this.
    pub fn fb(&self) -> &Fb {
        &self.fb
    }
}

impl super::GemHandle for DumbBuf {
    fn bo_handle(&self) -> u32 {
        self.handle
    }
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
}

/// Flags for DumbBuf creation.
///
/// TODO: find out what values these should be.
///
/// (DUNNO is Australian slang for "I don't know").
bitflags! {
    pub flags DumbBufFlags: u32 {
        // TODO: Find out what the actually flags are.
        const DUNNO = 1 << 0
    }
}
