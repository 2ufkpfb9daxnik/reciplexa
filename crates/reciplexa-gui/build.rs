//! Windows default stack is 1 MiB. Package Core typecheck/eval of Hybrid
//! Native v1 bodies needs more, and winit's event loop must stay on the
//! real process main thread — so bump the PE stack instead of spawning.

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if !target.contains("windows") {
        return;
    }
    // Must match `reciplexa::HOST_STACK_SIZE`.
    const STACK: u32 = 8 * 1024 * 1024;
    if target.contains("msvc") {
        println!("cargo:rustc-link-arg=/STACK:{STACK}");
    } else {
        println!("cargo:rustc-link-arg=-Wl,--stack,{STACK}");
    }
}
