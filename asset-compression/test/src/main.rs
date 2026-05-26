use asset_compression::{TargetColorFormat, rebuild_graphics_if_changed};

fn main () {
    // Normally this would go in build.rs
    rebuild_graphics_if_changed("./input", "./output", TargetColorFormat::Rgb565).unwrap();
}