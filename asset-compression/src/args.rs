// use log::error;
// use std::process::exit;

// pub struct Args {
//     pub assets_input_dir: String,
// }

// pub fn process_args(args: Vec<String>) -> Args {
//     if args.len() <= 1 {
//         error!("Path to graphics input must be provided. e.g. cargo run -- ../path/to/graphics");
//         exit(1);
//     } else if args.len() >= 3 {
//         error!("Too many arguments provided");
//         exit(1);
//     } else {
//         Args {
//             assets_input_dir: args[1].clone(),
//         }
//     }
// }
