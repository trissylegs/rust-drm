
extern crate drm;

fn main()
{
    let mut dev = drm::Device::first_card().unwrap();

    {
        println!("Set master!");
        let _master = match dev.set_master() {
            Err(err) => panic!("Failed to get master: {} (kind: {:?})", err, err.kind()),
            Ok(m) => m
        };
        println!("Got master");

        println!("Dropping master");
    }
    
    println!("Master should now be dropped.");
}
