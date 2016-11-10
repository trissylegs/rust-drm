
extern crate drm;
extern crate libc;

use std::thread;
use std::time::{Instant,Duration};
use drm::Device;
use drm::mode::*;

// ARGB8888 colors.
//                       BLUE, GREEN, RED, PADDING/ALPHA
const RED:     [u8; 4] = [0x00, 0x00, 0xff, 0xff];
const GREEN:   [u8; 4] = [0x00, 0xff, 0x00, 0xff];
const BLUE:    [u8; 4] = [0xff, 0x00, 0x00, 0xff];
const YELLOW:  [u8; 4] = [0x00, 0xff, 0xff, 0xff];
const MAGENTA: [u8; 4] = [0xff, 0x00, 0xff, 0xff];
const CYAN:    [u8; 4] = [0xff, 0xff, 0x00, 0xff];
const WHITE:   [u8; 4] = [0xff, 0xff, 0xff, 0xff];

fn main()
{
    // Fetch the first device.
    let mut dev = Device::first_card().expect("Failed to get a dri device");

    // Get the card resources.
    let res = Resources::get(&dev).unwrap();

    // Array of frame buffers being used.
    let mut buffers = Vec::new();
    // Connector/CRTC states to restore in exit.
    let mut restore = Vec::new();

    // Get control of the card.
    let master = dev.set_master().expect("Failed to set master");

    for &conn_id in res.connectors() {
        match Connector::get(&master, conn_id) {
            Err(err) => println!("Failed to get connector {:?}: {}", conn_id, err),
            Ok(conn) => {
                // Skip disconnected connectors.
                if conn.connection() != Connection::Connected { continue; }

                // Get the encoder of the connector.
                let encoder = match conn.encoder_id() {
                    None => continue,
                    Some(encoder_id) => Encoder::get(&master, encoder_id)
                        .expect("Failed to get encoder"),
                };

                // Get the CRTC associated with connector.
                let crtc = match encoder.crtc_id() {
                    None => continue,
                    Some(id) => Crtc::get(&master, id)
                        .expect("Failed to get crtc"),
                };

                // Get the current mode.
                let mode = match crtc.mode() {
                    None => continue,
                    Some(&mode) => mode,
                };

                // Create a frame buffer.
                let buf = DumbBuf::create(&master,
                                          mode.hdisplay as u32,
                                          mode.vdisplay as u32, 32, DUNNO)
                    .expect("Failed to create dumb buffer");

                // Put the frame buffer on the screen.
                master.set_crtc(crtc.id(), Some(buf.fb().id()), 0, 0,
                                &[conn.connector_id()], Some(&mode))
                    .expect("Failed to set mode");

                // FIME: What if two connectors are set to the same CRTC!

                // Save all this information.
                buffers.push(buf);
                restore.push((conn, crtc));
            }
        }
    }

    let colors = &[MAGENTA, CYAN, YELLOW, RED, GREEN, BLUE, WHITE];

    let start = Instant::now();
    let mut pixel_count = 0;
    // Pick a differnt color for each screen.
    for (buf, col) in buffers.iter_mut().zip(colors.into_iter().cycle())
    {
        // On debug this is really slow.
        for pixel in buf.as_mut().chunks_mut(4)
        {
            pixel[0] = col[0];
            pixel[1] = col[1];
            pixel[2] = col[2];
            pixel[3] = col[3];
        }
        pixel_count += buf.as_mut().len() / 4;
    }
    let elapsed = start.elapsed();
    println!("Render time = {}.{}", elapsed.as_secs(), elapsed.subsec_nanos() / 1000_000);
    println!("{} per pixel", (elapsed / pixel_count as u32).subsec_nanos());

    thread::sleep(Duration::new(3, 0));

    // Restore Connector, CRTC states.
    for (conn, crtc) in restore {
        let (x,y) = crtc.pos();
        master.set_crtc(crtc.id(), crtc.fb_id(),
                        x, y, &[conn.connector_id()],
                        crtc.mode())
            .unwrap_or_else(|err| println!("Error setting crtc: {}", err));
    }
    
    drop(master);
}
