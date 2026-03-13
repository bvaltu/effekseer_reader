use std::path::Path;

fn main() {
    let path = "/mnt/c/Users/midir/gamedev/projects/libs/effekseer/bevy_effekseer/assets/example/homing_laser.efkefc";
    let data = std::fs::read(path).expect("failed to read file");
    let effect = effekseer_reader::load_efkefc(&data).expect("failed to parse");
    
    println!("version: {}", effect.version);
    println!("target_location: {:?}", effect.target_location);
    println!("root children: {}", effect.root.children.len());
    
    fn dump_node(node: &effekseer_reader::types::node::EffectNode, depth: usize) {
        let indent = "  ".repeat(depth);
        println!("{}node_type: {:?}", indent, node.node_type);
        if let Some(ref params) = node.params {
            println!("{}  force_fields count: {}", indent, params.force_fields.len());
            for (i, ff) in params.force_fields.iter().enumerate() {
                println!("{}  force_field[{}]:", indent, i);
                println!("{}    field_type: {:?}", indent, ff.field_type);
                println!("{}    power: {}", indent, ff.power);
                println!("{}    position: {:?}", indent, ff.position);
                println!("{}    rotation: {:?}", indent, ff.rotation);
                println!("{}    type_params: {:?}", indent, ff.type_params);
                println!("{}    falloff: {:?}", indent, ff.falloff);
            }
        }
        for child in &node.children {
            dump_node(child, depth + 1);
        }
    }
    
    dump_node(&effect.root, 0);
}
