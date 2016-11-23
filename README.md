
# Rust DRM

Rust DRM provides a Rust interface to the Direct Rendering Manager API
in Linux.  This uses ioctl's to talk to a DRM driver in order to get the
current state of screens, perform mode setting, get vsync timing and
allocate dumb buffers.

(Nothing to do with Digital Rights Management.)

[Documentation](https://hornetblack.github.io/rust-drm/drm/index.html)

## To use

Add to Cargo.toml:

```
drm = { git = "https://github.com/HornetBlack/rust-drm.git" }
```

Add to your crate:

```
extern crate drm;
```

## Things to see and do:

* Listing availble cards:

```rust
use drm::Device;
for path in Device::cards().unwrap() {
    println!("{}", path.display());
    let mut dev = Device::open(&path).unwrap();
}
```

* Get the version information:

```rust
let version = dev.version();
println!("name: {}, description: {}", version.name().unwrap(), version.description().unwrap());
let (major, minor, patch) = version.number();
println!("version {}.{}.{}", major, minor, patch);
```

* Get resource information:

```rust
use drm::mode::*;
let res = dev.get_resources().unwrap();
for conn in res.connectors() {
    println!("{:#?}", dev.get(conn));
}
```

* Get vblank events:

```rust
use std::io::ErrorKind;
dev.request_vblank(12345, 0).unwrap();
loop {
    match dev.read_event() {
        // The actual message
        Ok(event) => { println!("{:?}", event); break }

        // Error handling
        Err(ref err) if err.kind() == ErrorKind::Interrupted { continue }
        Err(ref err) if err.kind() == ErrorKind::WouldBlock  { continue }
        Err(err) => panic!("Device::read_event: {}", err)
    }
}
```

* Create a "Dumb buffer".

```rust
use drm::mode::{DumbBuf, DUNNO};
let buf = DumbBuf::create(&dev, 1920, 1080, 32, DUNNO).unwrap();
```

* Put a frame buffer on a screen: (Requires some setup, see
`examples/magenta.rs` for a example)

```rust
let master = dev.set_master().unwrap();
master.set_crtc(crtc_id, Some(fb_id), 0, 0, &[conn_id], mode).unwrap();
```

## TODO:

- [ ] Better documentation.
- [ ] Capability information.
- [ ] Page flip.
- [ ] Clean up DumbBuf.
- [ ] Implement Cursors. I don't have hardware to test this.


