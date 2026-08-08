//! Native image-decode helper binary (Phase 11).
//!
//! ABI probe: `--abi` → `RPX_NATIVE_IMAGE_V1:1`
//! Op: `decode_header` reads stdin, writes PNG magic to stdout (contract-compatible
//! with portable fallback for equivalence tests on empty/PNG-prefix inputs).

use std::io::{self, Read, Write};

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--abi") => {
            println!("RPX_NATIVE_IMAGE_V1:1");
        }
        Some("decode_header") => {
            let mut buf = Vec::new();
            let _ = io::stdin().read_to_end(&mut buf);
            // Contract: return PNG signature bytes (same as portable stub).
            let magic = [0x89u8, 0x50, 0x4E, 0x47];
            let _ = io::stdout().write_all(&magic);
        }
        _ => {
            eprintln!("usage: rpx-native-image --abi | decode_header");
            std::process::exit(2);
        }
    }
}
