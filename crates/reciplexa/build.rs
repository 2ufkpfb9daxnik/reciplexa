//! Same 8 MiB PE stack as `reciplexa-gui` for package ingest on the CLI
//! process main thread (in addition to [`reciplexa::run_on_host_stack`]).

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
