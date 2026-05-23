use log::{error, info, warn};
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
use std::{
    fs::{self, DirEntry, ReadDir},
    path::Path,
    process::exit,
};

struct Args {
    assets_source_dir: String,
}

// #[derive(Debug)]
// struct AssetInfo {
//     path: DirEntry,
//     name: String,
//     width: u16,
//     height: u16,
//     asset_type: AssetType,
// }

// #[derive(Debug)]
// enum AssetType {
//     Static,
//     Animated(AnimationInfo),
// }

// #[derive(Debug)]
// struct AnimationInfo {
//     number_of_frames: u16,
// }

// #[derive(Debug)]
// enum AssetPath {
//     Static(StaticAsset),
//     Animated(AnimatedAsset),
// }

// #[derive(Debug)]
// struct StaticAsset {
//     name: String,
//     width: u16,
//     height: u16,
// }

// #[derive(Debug)]
// struct AnimatedAsset {
//     name: String,
//     width: u16,
//     height: u16,
//     number_of_frames: u16,
// }

// TODO add a target color depth (e.g. rgb565)

const FPS: u16 = 20;

fn main() {
    env_logger::init();
    let Args { assets_source_dir } = process_args(std::env::args().collect());

    let assets_source_dir = Path::new(&assets_source_dir);

    let mut assets_output_dir = assets_source_dir.parent().unwrap().to_path_buf();
    assets_output_dir.push("output");
    let assets_output_dir = Path::new(&assets_output_dir);

    prepare_output_directory(assets_output_dir);

    // let assets_source_dir = process_args(args);

    // for asset in assets_source_dir {

    // }

    // let asset_info = get_asset_info(asset_path);
}

fn process_args(args: Vec<String>) -> Args {
    if args.len() <= 1 {
        error!("Path to graphics input must be provided. e.g. cargo run -- ../path/to/graphics");
        exit(1);
    } else if args.len() >= 3 {
        error!("Unexpected argument provided");
        exit(1);
    } else {
        Args {
            assets_source_dir: args[1].clone(),
        }
        // match fs::read_dir(args[1].clone()) {
        //     Ok(_) => Args {
        //         assets_source_dir: args[1].clone(),
        //     },
        //     Err(_) => {
        //         error!("Unable to read provided path: {:?}", args[1]);
        //         exit(1);
        //     }
        // }
    }
}

fn prepare_output_directory(output_dir: &Path) {
    if fs::exists(output_dir).unwrap() {
        // Delete the existing directory
        fs::remove_dir_all(output_dir).unwrap()
    }

    fs::create_dir(output_dir).unwrap()

    // let exists: bool = match fs::exists(output_dir) {
    //     Ok(exists) => exists,
    //     Err(e) => ,
    // };
    // if fs::exists(output_dir) {

    // }
}

// TODO rename
// fn get_asset_info(asset_path: String) -> Vec<AssetInfo> {
//     let mut asset_info: Vec<AssetInfo> = Vec::new();

//     let root_dir = match fs::read_dir("../input") {
//         Ok(read_dir) => read_dir,
//         Err(_) => {
//             error!("Unable to read directory {:?}", asset_path);
//             exit(1);
//         }
//     };

//     for entry in root_dir {
//         let entry = entry.unwrap_or_else(|e| {
//             error!("Failed to read DirEntry: {:?}", e);
//             exit(1);
//         });

//         let file_type = entry.file_type().unwrap_or_else(|e| {
//             error!("Failed to determine file type of {:?}: {:?}", entry, e);
//             exit(1);
//         });

//         if file_type.is_dir() {
//             asset_info.push(get_animation_info(entry));
//         } else if file_type.is_file() {
//         } else {
//             error!("Unrecognised file type {:?}", file_type);
//             exit(1)
//         }
//     }

//     Ok(asset_list)
// }

// fn get_animation_info(dir: DirEntry) -> AssetInfo {
//     todo!()
// }

// fn get_static_asset_info(dir: DirEntry) -> AssetInfo {

// }
