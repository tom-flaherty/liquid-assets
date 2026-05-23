/// Expected structure for input directory:
/// 
/// input
/// ├── animation_name1
/// │   ├── frame1.png
/// │   ├── frame2.png
/// │   ├── frame3.png
/// │   ├── frame4.png
/// │   └── ...
/// ├── animation_name2
/// │   ├── frame1.png
/// │   ├── frame2.png
/// │   ├── frame3.png
/// │   ├── frame4.png
/// │   └── ...
/// ├── asset_name1.png
/// ├── asset_name2.png
/// └── asset_name3.png
/// 
/// Non-png files can be included, e.g. notes or a source gif. These will be ignored.
/// Directories cannot be included in the 

use std::fs;

fn main() -> std::io::Result<()> {
    get_names();
    Ok(())
}

/// TODO rename
fn get_names() {
    for entry in fs::read_dir("../input").unwrap() {
        println!("{:?}", entry);
    }
}
