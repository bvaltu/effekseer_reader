//! Debug helper: parse an .efkefc file with tracing enabled.
//!
//! Usage: RUST_LOG=debug cargo run --example debug_node -- path/to/effect.efkefc
//! Hex dump: cargo run --example debug_node -- path/to/effect.efkefc hexdump <offset> <len>

use std::env;
use std::fs;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let path = args.get(1).expect("Usage: debug_node <path> [hexdump <offset> <len>]");
    let data = fs::read(path).expect("Failed to read file");

    // Extract SKFE binary for hex dumping
    if args.get(2).map(|s| s.as_str()) == Some("hexdump") {
        let offset: usize = args
            .get(3)
            .expect("need offset")
            .parse()
            .expect("bad offset");
        let len: usize = args
            .get(4)
            .expect("need length")
            .parse()
            .expect("bad len");
        // Extract SKFE from EFKE container
        let skfe = extract_skfe(&data).expect("Failed to extract SKFE");
        let end = (offset + len).min(skfe.len());
        println!("SKFE hex dump at offset {}..{} (SKFE total len: {}):", offset, end, skfe.len());
        // Also show i32 interpretations
        if (end - offset) >= 4 {
            println!("--- i32 values ---");
            let mut p = offset;
            while p + 4 <= end {
                let v = i32::from_le_bytes([skfe[p], skfe[p+1], skfe[p+2], skfe[p+3]]);
                let vf = f32::from_le_bytes([skfe[p], skfe[p+1], skfe[p+2], skfe[p+3]]);
                println!("  pos {:>5}: {:>12} (0x{:08X}) float={:.6}", p, v, v as u32, vf);
                p += 4;
            }
            println!("--- hex ---");
        }
        for (i, chunk) in skfe[offset..end].chunks(16).enumerate() {
            let addr = offset + i * 16;
            print!("{:08x}: ", addr);
            for (j, byte) in chunk.iter().enumerate() {
                print!("{:02x}", byte);
                if j % 2 == 1 {
                    print!(" ");
                }
            }
            // Pad if short
            for j in chunk.len()..16 {
                print!("  ");
                if j % 2 == 1 {
                    print!(" ");
                }
            }
            print!(" ");
            for byte in chunk {
                if *byte >= 0x20 && *byte < 0x7f {
                    print!("{}", *byte as char);
                } else {
                    print!(".");
                }
            }
            println!();
        }
        return;
    }

    match effekseer_reader::load_efkefc(&data) {
        Ok(effect) => {
            println!(
                "SUCCESS: version={}, nodes={}",
                effect.version,
                count_nodes(&effect.root)
            );
        }
        Err(e) => {
            eprintln!("ERROR: {e}");
            std::process::exit(1);
        }
    }
}

fn count_nodes(node: &effekseer_reader::types::node::EffectNode) -> usize {
    1 + node.children.iter().map(count_nodes).sum::<usize>()
}

/// Extract BIN_ chunk data from EFKE container.
fn extract_skfe(data: &[u8]) -> Option<Vec<u8>> {
    // Search for "BIN_" chunk FourCC
    let mut pos = 8; // skip EFKE magic + version
    while pos + 8 <= data.len() {
        let fourcc = &data[pos..pos + 4];
        let size =
            u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
                as usize;
        pos += 8;
        if fourcc == b"BIN_" {
            let end = (pos + size).min(data.len());
            return Some(data[pos..end].to_vec());
        }
        pos += size;
    }
    None
}
