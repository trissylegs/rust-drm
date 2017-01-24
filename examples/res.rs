
extern crate drm;

use drm::Device;
use drm::mode::*;
use std::collections::BTreeSet;

fn main() {
    let mut dev = Device::first_card().unwrap();
    let res = Resources::get(&mut dev).unwrap();
    let min = res.min_size();
    let max = res.max_size();
    println!("min size: {}x{}", min.0, min.1);
    println!("max size: {}x{}", max.0, max.1);
    println!(" fb_ids:        {:?}", res.fbs());
    println!(" crtc_ids:      {:?}", res.crtcs());
    println!(" connector_ids: {:?}", res.connectors());
    println!(" encoder_ids:   {:?}", res.encoders());

    let mut props: BTreeSet<Id<Property>> = BTreeSet::new();
    
    for id in res.connectors() {
        match Connector::get(&mut dev, id.clone()) {
            Err(err) => println!("Error fetching connector {}: {}", id.as_u32(), err),
            Ok(conn) => {
                println!("{:#?}", conn);
                for (prop_id, _value) in conn.props() {
                    props.insert(prop_id);
                }
            }
        }
    }

    for prop_id in props {
        match Property::get(&mut dev, prop_id.clone()) {
            Ok(prop) =>
                println!("{:#?}", prop),
            Err(err) =>
                println!("Error getting prop (id = {}): {}",
                         prop_id.as_u32(), err),
        }
    }
    
    for id in res.crtcs() {
        match Crtc::get(&mut dev, id.clone()) {
            Err(err) => println!("Error fetching crtc({}): {}", id.as_u32(), err),
            Ok(crtc) => println!("{:#?}", crtc),
        }
    }

    for id in res.encoders() {
        match Encoder::get(&mut dev, id.clone()) {
            Err(err) => println!("Error fetching encoder({}): {}", id.as_u32(), err),
            Ok(enc) => println!("{:#?}", enc),
        }
    }

    for id in Plane::get_ids(&mut dev).unwrap() {
        match Plane::get(&mut dev, id.clone()) {
            Err(err) => println!("Error fetching plane({:?}): {}", id, err),
            Ok(plane) => println!("{:#?}", plane),
        }
    }
}
