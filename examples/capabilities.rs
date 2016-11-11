
extern crate drm;

use drm::Device;
use drm::Capability;

fn main() {
    for path in Device::cards().unwrap() {
        let dev = match Device::open(&path) {
            Ok(dev) => dev,
            Err(err) => { println!("Error opening {}: {}", path.display(), err);
                          continue }
        };
        println!("{}:", path.display());
        println!("  DumbBuffer         {}", dev.capability(Capability::DumbBuffer).unwrap());
        println!("  VblankHighCrtc     {}", dev.capability(Capability::VblankHighCrtc).unwrap());
        println!("  DumbPreferredDepth {}", dev.capability(Capability::DumbPreferredDepth).unwrap());
        println!("  DumbPreferShadow   {}", dev.capability(Capability::DumbPreferShadow).unwrap());
        println!("  Prime              {}", dev.capability(Capability::Prime).unwrap());
        println!("  TimestampMonotonic {}", dev.capability(Capability::TimestampMonotonic).unwrap());
        println!("  AsyncPageFlip      {}", dev.capability(Capability::AsyncPageFlip).unwrap());
        println!("  CursorWidth        {}", dev.capability(Capability::CursorWidth).unwrap());
        println!("  CursorHeight       {}", dev.capability(Capability::CursorHeight).unwrap());
        println!("  Addfb2Modifiers    {}", dev.capability(Capability::Addfb2Modifiers).unwrap());
    }
}
