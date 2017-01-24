
extern crate drm;

use drm::Device;
use drm::mode::*;

use std::collections::HashMap;

fn main()
{
    let dev = Device::first_card().expect("Failed to open a device.");

    let res = dev.get_resources().expect("Failed to get card resources");
    let mut prop_info = HashMap::new();

    for conn_id in res.connectors() {
        println!("Connector {}", conn_id.as_u32());
        print_props(&dev, *conn_id, &mut prop_info);
    }
    // for enc_id in res.encoders() {
    //     println!("Encoder {}", enc_id.as_u32());
    //     print_props(&dev, *enc_id, &mut prop_info);
    // }
    for crtc_id in res.crtcs() {
        println!("Crtc {}", crtc_id.as_u32());
        print_props(&dev, *crtc_id, &mut prop_info);
        
        // if let Some(fb_id) = dev.get(*crtc_id).ok().and_then(|crtc| crtc.fb_id()) {
        //     println!("Fb {} (current on Crtc {}", fb_id.as_u32(), crtc_id.as_u32());
        //     print_props(&dev, fb_id, &mut prop_info);
        // }
    }

    for plane_id in Plane::get_ids(&dev).unwrap() {
        println!("Plane {}", plane_id.as_u32());
        print_props(&dev, plane_id, &mut prop_info);
    }
}

fn print_props<T>(dev: &Device, id: Id<T>, prop_info: &mut HashMap<Id<Property>, Property>)
    where T: Resource
{
    let props = dev.get_object_props(id)
        .expect("Failed to get object properties");
    for (id, value) in props {
        let info = prop_info.entry(id).or_insert_with(|| {
            dev.get(id).expect(&format!("Failed to get info for prop {:?}", id))
        });
        println!("  Prop {}: {}", id.as_u32(), info.name());
        println!("info: {:?}", info);
        println!("    flags: {:?}", info.flags());
        if info.flags().contains(PROP_ENUM) {
            let enum_info = info.enums().iter().find(|e| e.value() == value as i64)
                .expect(&format!("Failed to find enum value {}", value));
            println!("    enum value: {:?}", enum_info.name());
        } else if info.flags().contains(PROP_BLOB) {
            println!("    id: {:?}", value);
        } else {
            println!("    value: {}", value);
        }
    }
}

