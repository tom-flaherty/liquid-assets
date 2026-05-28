use std::{fs, path::Path};

pub fn prepare_output_directory(output_dir: &Path) {
    if fs::exists(output_dir).unwrap() {
        // Delete the existing directory including its contents
        fs::remove_dir_all(output_dir).unwrap();
    }
    fs::create_dir(output_dir).unwrap()
}
