
extern crate drm;

use std::io::Result as IoResult;
use std::thread::sleep;
use std::time::Duration;

fn main() -> IoResult<()>
{
    let mut dev0 = drm::Device::first_card().unwrap();

    let dev = dev0.set_master()
        .map_err(|err| {
            eprintln!("Failed to set master: {:?}", err);
            err
        })?;

    let res = dev.get_resources()?;

    let connector = res.connectors().iter()

        .filter_map(|id| dev.get(*id).ok())
        
        .find(|conn| conn.encoder_id().is_some())
        
        .expect("No active connectors");

    let encoder_id = connector.encoder_id().unwrap();
    let encoder = dev.get(encoder_id)
        .expect("failed get encoder");        

    let crtc_id = encoder.crtc_id().unwrap();
    let crtc = dev.get(crtc_id)
        .expect("failed get crtc");

    let old_fbid = crtc.fb_id().expect("Currently no fb");
    
    let mode = crtc.mode().expect("mode")
        .clone();

    let mut buffer = drm::mode::DumbBuf::create_with_depth(
        &dev,
        mode.hdisplay as u32, mode.vdisplay as u32, 32, 32)
        .expect("creating buffer");

    dev.set_crtc(crtc.id(),  Some(buffer.fb().id()),
                 0, 0,
                 &[ connector.id() ],
                 Some(&mode))
        .expect("set_crtc 1");
    
    fill_buffer(&mut buffer);

    sleep(Duration::new(1, 0));

    dev.set_crtc(crtc.id(), Some(old_fbid),
                 0, 0,
                 &[ connector.id() ],
                 Some(&mode))
        .expect("set_crtc 1");
    
    Ok(())
}


fn fill_buffer<B:AsMut<[u32]>>(mut buffer_ref: B) {
    let mut buffer = buffer_ref.as_mut();

    for p in buffer.iter_mut() {
        *p = 0xffff00ff;
    }
}
