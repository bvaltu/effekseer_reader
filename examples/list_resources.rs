//! List all resource paths referenced by an effect file.
//!
//! Usage: cargo run --example list_resources -- path/to/effect.efkefc

use std::env;
use std::fs;

use effekseer_reader::load_efkefc;

fn main() {
    let path = env::args().nth(1).expect("Usage: list_resources <path>");
    let data = fs::read(&path).expect("Failed to read file");
    let effect = load_efkefc(&data).expect("Failed to parse");

    print_section("Color Textures", &effect.color_images);
    print_section("Normal Textures", &effect.normal_images);
    print_section("Distortion Textures", &effect.distortion_images);
    print_section("Sounds", &effect.sounds);
    print_section("Models", &effect.models);
    print_section("Materials", &effect.materials);
    print_section("Curves", &effect.curves);
}

fn print_section(label: &str, paths: &[String]) {
    if paths.is_empty() {
        return;
    }
    println!("{label} ({}):", paths.len());
    for path in paths {
        println!("  {path}");
    }
    println!();
}
