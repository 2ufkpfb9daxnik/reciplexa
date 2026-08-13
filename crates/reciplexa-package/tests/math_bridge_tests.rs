//! HC4: `estimate_package_math_main` over `examples/pkg_math.rpx`.

use std::path::PathBuf;

use reciplexa_package::{estimate_package_math_main, LocalPackageIndex};

fn workspace_packages() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages")
}

#[test]
fn estimate_package_math_main_pkg_math_demo_tree() {
    std::thread::Builder::new()
        .name("math-main-box".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
            let entry =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_math.rpx");
            let box_ = estimate_package_math_main(&entry, &idx).expect("math main box");
            assert!(
                box_.width > 0.0 && box_.total_height() > 0.0,
                "expected non-empty MathBox, got {box_:?}"
            );
        })
        .expect("spawn")
        .join()
        .expect("join");
}
