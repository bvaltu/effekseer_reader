//! Parse an .efkefc file and print a summary of its contents.
//!
//! Usage: cargo run --example dump_effect -- path/to/effect.efkefc

use std::env;
use std::fs;

use effekseer_reader::load_efkefc;
use effekseer_reader::types::enums::EffectNodeType;
use effekseer_reader::types::node::{EffectNode, RendererVariant};

fn main() {
    let path = env::args().nth(1).expect("Usage: dump_effect <path>");
    let data = fs::read(&path).expect("Failed to read file");
    let effect = load_efkefc(&data).expect("Failed to parse");

    println!("Effect Summary");
    println!("==============");
    println!("Version:       {}", effect.version);
    println!("Magnification: {}", effect.magnification);
    println!("Random Seed:   {}", effect.random_seed);

    if let Some(ref culling) = effect.culling {
        println!(
            "Culling:       {:?} (radius={}, pos=[{},{},{}])",
            culling.shape,
            culling.radius,
            culling.location.x,
            culling.location.y,
            culling.location.z
        );
    }

    if let Some(lod) = effect.lod_distances {
        println!("LOD Distances: [{}, {}, {}]", lod[0], lod[1], lod[2]);
    }

    println!();
    println!("Resources");
    println!("---------");
    println!("Color Textures:      {}", effect.color_images.len());
    println!("Normal Textures:     {}", effect.normal_images.len());
    println!("Distortion Textures: {}", effect.distortion_images.len());
    println!("Sounds:              {}", effect.sounds.len());
    println!("Models:              {}", effect.models.len());
    println!("Materials:           {}", effect.materials.len());
    println!("Curves:              {}", effect.curves.len());
    println!("Procedural Models:   {}", effect.procedural_models.len());
    println!("Dynamic Inputs:      {}", effect.dynamic_inputs.len());
    println!("Dynamic Equations:   {}", effect.dynamic_equations.len());

    println!();
    println!("Node Tree");
    println!("---------");
    let (total, by_type) = count_nodes(&effect.root);
    println!("Total nodes: {total}");
    for (ty, count) in &by_type {
        println!("  {ty}: {count}");
    }

    println!();
    print_tree(&effect.root, 0);
}

fn count_nodes(node: &EffectNode) -> (usize, Vec<(String, usize)>) {
    let mut counts = std::collections::HashMap::new();
    count_recursive(node, &mut counts);
    let total: usize = counts.values().sum();
    let mut by_type: Vec<(String, usize)> = counts.into_iter().collect();
    by_type.sort_by_key(|(name, _)| name.clone());
    (total, by_type)
}

fn count_recursive(node: &EffectNode, counts: &mut std::collections::HashMap<String, usize>) {
    *counts.entry(format!("{:?}", node.node_type)).or_default() += 1;
    for child in &node.children {
        count_recursive(child, counts);
    }
}

fn print_tree(node: &EffectNode, depth: usize) {
    let indent = "  ".repeat(depth);
    let renderer = node
        .params
        .as_ref()
        .map(|p| match &p.renderer {
            RendererVariant::None => "None",
            RendererVariant::Sprite(_) => "Sprite",
            RendererVariant::Ribbon(_) => "Ribbon",
            RendererVariant::Ring(_) => "Ring",
            RendererVariant::Model(_) => "Model",
            RendererVariant::Track(_) => "Track",
            _ => "Unknown",
        })
        .unwrap_or("-");

    let rendered = node
        .params
        .as_ref()
        .map(|p| if p.is_rendered { "rendered" } else { "hidden" })
        .unwrap_or("-");

    let type_name = match node.node_type {
        EffectNodeType::Root => "Root",
        EffectNodeType::NoneType => "NoneType",
        EffectNodeType::Sprite => "Sprite",
        EffectNodeType::Ribbon => "Ribbon",
        EffectNodeType::Ring => "Ring",
        EffectNodeType::Model => "Model",
        EffectNodeType::Track => "Track",
        _ => "Unknown",
    };

    println!(
        "{indent}{type_name} (renderer={renderer}, {rendered}, children={})",
        node.children.len()
    );

    for child in &node.children {
        print_tree(child, depth + 1);
    }
}
