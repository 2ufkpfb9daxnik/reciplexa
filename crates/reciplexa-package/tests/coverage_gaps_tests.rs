//! Extra region-coverage for lockfile / rpxm / workspace / rpi / load error paths.

use std::fs;
use std::path::PathBuf;

use reciplexa_package::{
    parse_rpi_exports, parse_rpxm, parse_workspace_rpxm, DependencySpec, LocalPackageIndex,
    Lockfile, PackageLoadError, PackageManifest, RpxmError,
};

fn scratch() -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.tmp/pkg-cov-gaps");
    let _ = fs::create_dir_all(&base);
    let dir = base.join(format!(
        "t-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_pkg(root: &std::path::Path, name: &str, body: &str) -> PathBuf {
    let pkg = root.join(name);
    fs::create_dir_all(&pkg).unwrap();
    fs::write(pkg.join("package.rpxm"), body).unwrap();
    pkg
}

// --- lockfile ---

#[test]
fn lockfile_from_graph_uses_formal_package_name() {
    let m = PackageManifest {
        name: "app".into(),
        version: "1.0".into(),
        dependencies: vec![DependencySpec {
            name: "alias".into(),
            version_req: "1".into(),
            path: Some("../lib".into()),
            package: Some("formal-lib".into()),
        }],
        entry: "main.rpx".into(),
        ..Default::default()
    };
    let lf = Lockfile::from_graph(&[m]);
    assert_eq!(lf.packages[0].dependencies, vec!["formal-lib".to_string()]);
}

#[test]
fn lockfile_from_consumer_workspace_source_and_nested_path_deps() {
    let consumer = PackageManifest {
        name: "app".into(),
        version: "1.0".into(),
        dependencies: vec![DependencySpec {
            name: "g".into(),
            version_req: "*".into(),
            path: Some("../graphics".into()),
            package: Some("graphics".into()),
        }],
        entry: "main.rpx".into(),
        ..Default::default()
    };
    let dep_manifest = PackageManifest {
        name: "graphics".into(),
        version: "0.1.0".into(),
        dependencies: vec![
            DependencySpec {
                name: "util".into(),
                version_req: "1".into(),
                path: Some("../util".into()),
                package: None,
            },
            DependencySpec {
                name: "reg".into(),
                version_req: "1".into(),
                path: None,
                package: None,
            },
        ],
        entry: String::new(),
        public_modules: vec!["shapes".into()],
        ..Default::default()
    };
    // No path → workspace source branch in from_consumer.
    let registry_only = DependencySpec {
        name: "g".into(),
        version_req: "*".into(),
        path: None,
        package: Some("graphics".into()),
    };
    let lf = Lockfile::from_consumer(&consumer, &[(&registry_only, &dep_manifest)]);
    let g = lf.packages.iter().find(|p| p.name == "graphics").unwrap();
    assert_eq!(g.source, "workspace");
    assert_eq!(g.dependencies, vec!["util".to_string()]);

    let with_path = DependencySpec {
        name: "g".into(),
        version_req: "0.1.0".into(),
        path: Some("../graphics".into()),
        package: Some("graphics".into()),
    };
    let lf2 = Lockfile::from_consumer(&consumer, &[(&with_path, &dep_manifest)]);
    let g2 = lf2.packages.iter().find(|p| p.name == "graphics").unwrap();
    assert!(g2.source.starts_with("path:"));
}

#[test]
fn lockfile_consistency_error_paths() {
    let consumer = PackageManifest {
        name: "app".into(),
        version: "1".into(),
        dependencies: vec![
            DependencySpec {
                name: "reg".into(),
                version_req: "1".into(),
                path: None,
                package: None,
            },
            DependencySpec {
                name: "g".into(),
                version_req: "0.2.0".into(),
                path: Some("../g".into()),
                package: Some("graphics".into()),
            },
        ],
        entry: "m".into(),
        ..Default::default()
    };

    let missing = Lockfile { packages: vec![] };
    let err = missing.is_consistent_with_consumer(&consumer).unwrap_err();
    assert!(err.contains("missing `graphics`"));

    let wrong_source = Lockfile {
        packages: vec![reciplexa_package::LockedPackage {
            name: "graphics".into(),
            version: "0.2.0".into(),
            source: "workspace".into(),
            dependencies: vec![],
        }],
    };
    let err = wrong_source
        .is_consistent_with_consumer(&consumer)
        .unwrap_err();
    assert!(err.contains("expected path source"));

    let version_mismatch = Lockfile {
        packages: vec![reciplexa_package::LockedPackage {
            name: "graphics".into(),
            version: "0.1.0".into(),
            source: "path:../g".into(),
            dependencies: vec![],
        }],
    };
    let err = version_mismatch
        .is_consistent_with_consumer(&consumer)
        .unwrap_err();
    assert!(err.contains("locked as"));
}

#[test]
fn lockfile_read_write_io_errors() {
    let lf = Lockfile { packages: vec![] };
    let err = lf
        .write_rpx_lock(PathBuf::from("/definitely/not/a/writable/path/rpx.lock"))
        .unwrap_err();
    assert!(err.contains("write lockfile"));

    let err = Lockfile::read_rpx_lock(PathBuf::from("/no/such/rpx.lock")).unwrap_err();
    assert!(err.contains("read lockfile"));

    let dir = scratch();
    let bad = dir.join("bad.lock");
    fs::write(&bad, "{not-json").unwrap();
    assert!(Lockfile::read_rpx_lock(&bad).is_err());
}

// --- workspace ---

#[test]
fn workspace_error_and_edge_paths() {
    assert_eq!(parse_workspace_rpxm(""), Err(RpxmError::Empty));
    assert_eq!(
        parse_workspace_rpxm("   (// only)\n"),
        Err(RpxmError::Empty)
    );

    let err = parse_workspace_rpxm("(workspace format-version x (members \"a\"))").unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));

    let err = parse_workspace_rpxm("(workspace format-version 2 (members \"a\"))").unwrap_err();
    assert_eq!(err, RpxmError::UnsupportedFormatVersion(2));

    let err = parse_workspace_rpxm("(workspace (members \"\"))").unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));

    let err = parse_workspace_rpxm("(workspace (members \"a\"").unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));

    let err = parse_workspace_rpxm("(workspace mystery 1 (members \"a\"))").unwrap_err();
    assert!(matches!(err, RpxmError::UnknownField(_)));
}

// --- rpxm ---

#[test]
fn rpxm_dd001_roots_and_entry_points() {
    let m = parse_rpxm(
        r#"(package app
  format-version 1
  version "1.0.0"
  source-root "src"
  interface-root "interface"
  resource-root "assets"
  (entry-points cli tool)
  (public-modules main))"#,
    )
    .unwrap();
    assert_eq!(m.interface_root.as_deref(), Some("interface"));
    assert_eq!(m.resource_root, "assets");
    assert_eq!(m.entry_points, vec!["cli".to_string(), "tool".to_string()]);
    assert_eq!(m.entry, "cli.rpx");
}

#[test]
fn rpxm_invalid_format_version_token() {
    let err = parse_rpxm(
        r#"(package app
  format-version nope
  version "1.0.0"
  (public-modules main))"#,
    )
    .unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));
}

#[test]
fn rpxm_ident_list_and_dependencies_errors() {
    let err = parse_rpxm(
        r#"(package app
  format-version 1
  version "1"
  (public-modules shapes (nested)))"#,
    )
    .unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));

    let err = parse_rpxm(
        r#"(package app
  format-version 1
  version "1"
  (public-modules shapes"#,
    )
    .unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));

    let err = parse_rpxm(
        r#"(package app
  format-version 1
  version "1"
  (public-modules main)
  (dependencies bare))"#,
    )
    .unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));

    let err = parse_rpxm(
        r#"(package app
  format-version 1
  version "1"
  (public-modules main)
  (dependencies
    (g package graphics version "1" path "../g")"#,
    )
    .unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));
}

#[test]
fn rpxm_dependency_nested_group_and_missing_version() {
    let m = parse_rpxm(
        r#"(package app
  format-version 1
  version "1"
  (public-modules main)
  (dependencies
    (g package graphics version "0.1" (extra nested) path "../g")))"#,
    )
    .unwrap();
    assert_eq!(m.dependencies[0].path.as_deref(), Some("../g"));

    let err = parse_rpxm(
        r#"(package app
  format-version 1
  version "1"
  (public-modules main)
  (dependencies
    (g package graphics path "../g")))"#,
    )
    .unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));
}

#[test]
fn rpxm_unknown_flat_field_and_comment_string() {
    let err = parse_rpxm(
        r#"(package app
  format-version 1
  version "1"
  mystery "x"
  (public-modules main))"#,
    )
    .unwrap_err();
    assert!(matches!(err, RpxmError::UnknownField(_)));

    // Structured comment containing a string exercises the comment lexer branch.
    let m = parse_rpxm(
        r#"(// "quoted inside comment")
(package
  (name demo)
  (version 0.1.0)
  (entry main.rpx))"#,
    )
    .unwrap();
    assert_eq!(m.name, "demo");
}

// --- rpi ---

#[test]
fn rpi_named_fn_anonymous_fn_and_errors() {
    let exports = parse_rpi_exports(
        r#"(val circle)
(fn area (r) (* 3 r r))
(fn (anon-params) 1)
(type point)
(unknown-form x)
"foo"
(val circle)
"#,
    )
    .unwrap();
    assert_eq!(exports, vec!["circle", "area", "point"]);

    let err = parse_rpi_exports("(val)").unwrap_err();
    assert!(matches!(err, PackageLoadError::Interface(_)));

    let err = parse_rpi_exports("(val (").unwrap_err();
    assert!(matches!(err, PackageLoadError::Interface(_)));

    let err = parse_rpi_exports("(type").unwrap_err();
    assert!(matches!(err, PackageLoadError::Interface(_)));
}

// --- load ---

#[test]
fn package_load_error_display_arms() {
    let io = PackageLoadError::Io("io".into());
    assert_eq!(io.to_string(), "io");
    let nf = PackageLoadError::NotFound("nf".into());
    assert_eq!(nf.to_string(), "nf");
    let amb = PackageLoadError::Ambiguous("a".into());
    assert_eq!(amb.to_string(), "a");
    let iface = PackageLoadError::Interface("i".into());
    assert_eq!(iface.to_string(), "i");
    let man = PackageLoadError::Manifest(RpxmError::Empty);
    assert!(man.to_string().contains("manifest"));
    let mod_err = PackageLoadError::Module(reciplexa_bind::ModuleError {
        message: "m".into(),
    });
    assert_eq!(mod_err.to_string(), "m");
}

#[test]
fn discover_skips_nondir_and_detects_duplicate() {
    let root = scratch();
    fs::write(root.join("not-a-dir.txt"), "x").unwrap();
    write_pkg(
        &root,
        "a",
        r#"(package shared
  format-version 1
  version "1"
  (public-modules main))"#,
    );
    let root2 = scratch();
    write_pkg(
        &root2,
        "b",
        r#"(package shared
  format-version 1
  version "1"
  (public-modules main))"#,
    );
    // Non-dir search root is skipped.
    let idx = LocalPackageIndex::discover(&[root.join("missing"), root.clone()]).unwrap();
    assert!(idx.package_names().any(|n| n == "shared"));

    let err = LocalPackageIndex::discover(&[root.clone(), root2.clone()]).unwrap_err();
    assert!(matches!(err, PackageLoadError::Ambiguous(_)));
}

#[test]
fn register_path_dep_error_paths() {
    let root = scratch();
    let consumer = root.join("app");
    fs::create_dir_all(consumer.join("src")).unwrap();
    fs::write(
        consumer.join("package.rpxm"),
        r#"(package app
  format-version 1
  version "1"
  (public-modules main)
  (dependencies
    (g package graphics version "0.1.0" path "../graphics")))"#,
    )
    .unwrap();

    let mut idx = LocalPackageIndex::discover(&[scratch()]).unwrap();
    let man = parse_rpxm(&fs::read_to_string(consumer.join("package.rpxm")).unwrap()).unwrap();
    let err = idx.register_path_dependencies(&consumer, &man).unwrap_err();
    assert!(matches!(err, PackageLoadError::NotFound(_)));

    // Wrong formal name
    write_pkg(
        &root,
        "graphics",
        r#"(package other
  format-version 1
  version "0.1.0"
  (public-modules shapes))"#,
    );
    let err = idx.register_path_dependencies(&consumer, &man).unwrap_err();
    assert!(
        err.to_string().contains("expects package `graphics`"),
        "{err}"
    );

    // Version mismatch
    fs::write(
        root.join("graphics/package.rpxm"),
        r#"(package graphics
  format-version 1
  version "9.9.9"
  (public-modules shapes))"#,
    )
    .unwrap();
    let err = idx.register_path_dependencies(&consumer, &man).unwrap_err();
    assert!(err.to_string().contains("wants version"), "{err}");
}

#[test]
fn resolve_import_public_module_edges() {
    let root = scratch();
    write_pkg(
        &root,
        "graphics",
        r#"(package graphics
  format-version 1
  version "0.1.0"
  source-root "src"
  (public-modules shapes))"#,
    );
    fs::create_dir_all(root.join("graphics/src")).unwrap();
    fs::write(root.join("graphics/src/shapes.rpx"), "(val circle 1)\n").unwrap();

    let idx = LocalPackageIndex::discover(&[&root]).unwrap();
    // Bare package name → first public module.
    let resolved = idx.resolve_import("graphics").unwrap();
    assert_eq!(resolved.0, "graphics");
    assert!(resolved.1.contains("circle"));

    let err = idx.resolve_import("graphics/hidden").unwrap_err();
    assert!(err.to_string().contains("not public"));

    let err = idx.resolve_import("missing/x").unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[test]
fn resolve_bare_package_same_named_public_and_no_public() {
    let root = scratch();
    write_pkg(
        &root,
        "graphics",
        r#"(package graphics
  format-version 1
  version "0.1.0"
  source-root "src"
  (public-modules graphics shapes))"#,
    );
    fs::create_dir_all(root.join("graphics/src")).unwrap();
    fs::write(root.join("graphics/src/graphics.rpx"), "(val g 1)\n").unwrap();
    fs::write(root.join("graphics/src/shapes.rpx"), "(val s 1)\n").unwrap();
    let idx = LocalPackageIndex::discover(&[&root]).unwrap();
    let (name, src) = idx.resolve_import("graphics").unwrap();
    assert_eq!(name, "graphics");
    assert!(src.contains("(val g"));

    let root2 = scratch();
    write_pkg(
        &root2,
        "empty",
        r#"(package empty
  format-version 1
  version "1"
  (entry-points main))"#,
    );
    let idx2 = LocalPackageIndex::discover(&[&root2]).unwrap();
    let err = idx2.resolve_import("empty").unwrap_err();
    assert!(err.to_string().contains("no public modules"));
}

#[test]
fn load_module_tree_path_not_found() {
    let idx = LocalPackageIndex::discover(&[scratch()]).unwrap();
    let err = reciplexa_package::load_module_tree_with_packages(
        PathBuf::from("/no/such/entry.rpx"),
        &idx,
    )
    .unwrap_err();
    assert!(matches!(err, PackageLoadError::Io(_)));
}

#[cfg(windows)]
fn exclusive_open(path: &std::path::Path) -> std::fs::File {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .open(path)
        .expect("exclusive open")
}

#[test]
fn from_rpxm_error_and_discover_manifest_parse_err() {
    let _ = PackageLoadError::from(RpxmError::Empty);
    let _ = PackageLoadError::from(RpxmError::MissingName);

    let root = scratch();
    let bad = root.join("badpkg");
    fs::create_dir_all(&bad).unwrap();
    fs::write(bad.join("package.rpxm"), "(package").unwrap();
    let err = LocalPackageIndex::discover(&[&root]).unwrap_err();
    assert!(matches!(err, PackageLoadError::Manifest(_)));
}

#[test]
fn discover_skips_dir_without_manifest_and_io_on_locked_manifest() {
    let root = scratch();
    fs::create_dir_all(root.join("empty-child")).unwrap();
    write_pkg(
        &root,
        "ok",
        r#"(package ok
  format-version 1
  version "1"
  (public-modules main))"#,
    );
    let idx = LocalPackageIndex::discover(&[&root]).unwrap();
    assert!(idx.package_names().any(|n| n == "ok"));

    #[cfg(windows)]
    {
        let locked_root = scratch();
        let pkg = locked_root.join("locked");
        fs::create_dir_all(&pkg).unwrap();
        let man = pkg.join("package.rpxm");
        fs::write(
            &man,
            r#"(package locked
  format-version 1
  version "1"
  (public-modules main))"#,
        )
        .unwrap();
        let _guard = exclusive_open(&man);
        let err = LocalPackageIndex::discover(&[&locked_root]).unwrap_err();
        assert!(matches!(err, PackageLoadError::Io(_)), "{err}");
    }
}

#[test]
fn register_path_dep_skip_registry_only_and_reuse_discovered() {
    let root = scratch();
    write_pkg(
        &root,
        "graphics",
        r#"(package graphics
  format-version 1
  version "0.1.0"
  source-root "src"
  (public-modules shapes))"#,
    );
    fs::create_dir_all(root.join("graphics/src")).unwrap();
    fs::write(root.join("graphics/src/shapes.rpx"), "(val circle 1)\n").unwrap();

    let consumer = root.join("app");
    fs::create_dir_all(&consumer).unwrap();
    fs::write(
        consumer.join("package.rpxm"),
        r#"(package app
  format-version 1
  version "1"
  (public-modules main)
  (dependencies
    (reg package registry-only version "1")
    (g package graphics version "0.1.0" path "../graphics")))"#,
    )
    .unwrap();

    let mut idx = LocalPackageIndex::discover(&[&root]).unwrap();
    let man = parse_rpxm(&fs::read_to_string(consumer.join("package.rpxm")).unwrap()).unwrap();
    idx.register_path_dependencies(&consumer, &man).unwrap();
    assert_eq!(idx.resolve_alias("g"), "graphics");
    // Registry-only dep has no path → skipped; alias for reg must not exist.
    assert_eq!(idx.resolve_alias("reg"), "reg");

    #[cfg(windows)]
    {
        let root2 = scratch();
        let consumer2 = root2.join("app");
        fs::create_dir_all(&consumer2).unwrap();
        let dep = root2.join("dep");
        fs::create_dir_all(&dep).unwrap();
        let man_path = dep.join("package.rpxm");
        fs::write(
            &man_path,
            r#"(package dep
  format-version 1
  version "1"
  (public-modules main))"#,
        )
        .unwrap();
        fs::write(
            consumer2.join("package.rpxm"),
            r#"(package app
  format-version 1
  version "1"
  (public-modules main)
  (dependencies
    (d package dep version "1" path "../dep")))"#,
        )
        .unwrap();
        let mut idx2 = LocalPackageIndex::discover(&[scratch()]).unwrap();
        let man2 =
            parse_rpxm(&fs::read_to_string(consumer2.join("package.rpxm")).unwrap()).unwrap();
        let _guard = exclusive_open(&man_path);
        let err = idx2
            .register_path_dependencies(&consumer2, &man2)
            .unwrap_err();
        assert!(matches!(err, PackageLoadError::Io(_)), "{err}");
    }
}

#[test]
fn discover_with_consumer_parse_and_io_errors() {
    let root = scratch();
    let consumer = root.join("app");
    fs::create_dir_all(&consumer).unwrap();
    fs::write(consumer.join("package.rpxm"), "(package").unwrap();
    let err =
        LocalPackageIndex::discover_with_consumer(&[root.join("pkgs")], &consumer).unwrap_err();
    assert!(matches!(err, PackageLoadError::Manifest(_)));

    #[cfg(windows)]
    {
        let root2 = scratch();
        let consumer2 = root2.join("app");
        fs::create_dir_all(&consumer2).unwrap();
        let man = consumer2.join("package.rpxm");
        fs::write(
            &man,
            r#"(package app
  format-version 1
  version "1"
  (public-modules main))"#,
        )
        .unwrap();
        let _guard = exclusive_open(&man);
        let err = LocalPackageIndex::discover_with_consumer(&[root2.join("pkgs")], &consumer2)
            .unwrap_err();
        assert!(matches!(err, PackageLoadError::Io(_)), "{err}");
    }
}

#[test]
fn resolve_import_missing_interface_and_module_io() {
    let root = scratch();
    write_pkg(
        &root,
        "graphics",
        r#"(package graphics
  format-version 1
  version "0.1.0"
  source-root "src"
  interface-root "interface"
  (public-modules shapes))"#,
    );
    fs::create_dir_all(root.join("graphics/src")).unwrap();
    fs::write(root.join("graphics/src/shapes.rpx"), "(val circle 1)\n").unwrap();
    // interface-root declared but .rpi missing
    let idx = LocalPackageIndex::discover(&[&root]).unwrap();
    let err = idx.resolve_import("graphics/shapes").unwrap_err();
    assert!(err.to_string().contains("missing interface"), "{err}");

    fs::create_dir_all(root.join("graphics/interface")).unwrap();
    let rpi = root.join("graphics/interface/shapes.rpi");
    fs::write(&rpi, "(val circle)\n").unwrap();

    #[cfg(windows)]
    {
        let _guard = exclusive_open(&rpi);
        let err = idx.resolve_import_detailed("graphics/shapes").unwrap_err();
        assert!(matches!(err, PackageLoadError::Io(_)), "{err}");
    }

    // Missing module source file
    let root2 = scratch();
    write_pkg(
        &root2,
        "lib",
        r#"(package lib
  format-version 1
  version "1"
  source-root "src"
  (public-modules shapes))"#,
    );
    fs::create_dir_all(root2.join("lib/src")).unwrap();
    let idx2 = LocalPackageIndex::discover(&[&root2]).unwrap();
    let err = idx2.resolve_import("lib/shapes").unwrap_err();
    assert!(matches!(err, PackageLoadError::Io(_)), "{err}");
}

#[test]
fn lock_consumer_skips_registry_and_errors_when_missing() {
    let root = scratch();
    write_pkg(
        &root,
        "graphics",
        r#"(package graphics
  format-version 1
  version "0.1.0"
  (public-modules shapes))"#,
    );
    let mut idx = LocalPackageIndex::discover(&[&root]).unwrap();
    let consumer_root = root.join("app");
    fs::create_dir_all(&consumer_root).unwrap();
    fs::write(
        consumer_root.join("package.rpxm"),
        r#"(package app
  format-version 1
  version "1"
  (public-modules main)
  (dependencies
    (reg package registry-only version "1")
    (g package graphics version "0.1.0" path "../graphics")))"#,
    )
    .unwrap();
    let consumer =
        parse_rpxm(&fs::read_to_string(consumer_root.join("package.rpxm")).unwrap()).unwrap();
    idx.register_path_dependencies(&consumer_root, &consumer)
        .unwrap();
    let lock = idx.lock_consumer(&consumer).unwrap();
    assert!(lock.packages.iter().any(|p| p.name == "graphics"));
    // Registry-only deps are skipped by lock_consumer (no path).
    assert!(!lock.packages.iter().any(|p| p.name == "registry-only"));

    let missing = PackageManifest {
        name: "app".into(),
        version: "1".into(),
        dependencies: vec![DependencySpec {
            name: "gone".into(),
            version_req: "1".into(),
            path: Some("../gone".into()),
            package: Some("gone".into()),
        }],
        entry: "main.rpx".into(),
        ..Default::default()
    };
    let err = idx.lock_consumer(&missing).unwrap_err();
    assert!(matches!(err, PackageLoadError::NotFound(_)));
}

#[test]
fn load_module_tree_directory_self_import_sibling_and_packages() {
    let root = scratch();
    // Directory mode delegates to bind::load_module_tree.
    let dir_entry = root.join("moddir");
    fs::create_dir_all(&dir_entry).unwrap();
    fs::write(dir_entry.join("a.rpx"), "(val x 1)\n").unwrap();
    let idx = LocalPackageIndex::discover(&[scratch()]).unwrap();
    let units = reciplexa_package::load_module_tree_with_packages(&dir_entry, &idx).unwrap();
    assert!(units.iter().any(|(n, _)| n == "a"));

    // Self-import error
    let entry = root.join("self.rpx");
    fs::write(&entry, "(import self)\n(val main 1)\n").unwrap();
    let err = reciplexa_package::load_module_tree_with_packages(&entry, &idx).unwrap_err();
    assert!(matches!(err, PackageLoadError::Module(_)));

    // Sibling module + package import; revisit name for cycle-skip (`seen`)
    write_pkg(
        &root,
        "graphics",
        r#"(package graphics
  format-version 1
  version "0.1.0"
  source-root "src"
  (public-modules shapes))"#,
    );
    fs::create_dir_all(root.join("graphics/src")).unwrap();
    fs::write(root.join("graphics/src/shapes.rpx"), "(val circle 1)\n").unwrap();
    let idx = LocalPackageIndex::discover(&[&root]).unwrap();
    fs::write(root.join("sib.rpx"), "(val helper 2)\n").unwrap();
    let entry2 = root.join("app.rpx");
    fs::write(
        &entry2,
        r#"(import sib)
(import graphics/shapes)
(val main 1)
"#,
    )
    .unwrap();
    let units = reciplexa_package::load_module_tree_with_packages(&entry2, &idx).unwrap();
    assert!(units.iter().any(|(n, _)| n == "sib"));
    assert!(units.iter().any(|(n, _)| n == "graphics/shapes"));

    // elaborate_with_packages covers the interface overlay path (no rpi here).
    let units = reciplexa_package::elaborate_with_packages(&entry2, &idx).unwrap();
    assert!(!units.is_empty());

    #[cfg(windows)]
    {
        let entry3 = root.join("locked_entry.rpx");
        fs::write(&entry3, "(val main 1)\n").unwrap();
        {
            let _guard = exclusive_open(&entry3);
            let err = reciplexa_package::load_module_tree_with_packages(&entry3, &idx).unwrap_err();
            assert!(matches!(err, PackageLoadError::Io(_)), "{err}");
        }

        // Sibling file locked → Io while loading unit source
        let sib_locked = root.join("helpers.rpx");
        fs::write(&sib_locked, "(val s 1)\n").unwrap();
        let entry4 = root.join("uses_helpers.rpx");
        fs::write(&entry4, "(import helpers)\n(val main 1)\n").unwrap();
        // Sanity: unlocked load succeeds.
        reciplexa_package::load_module_tree_with_packages(&entry4, &idx).unwrap();
        let _guard2 = exclusive_open(&sib_locked);
        let err = reciplexa_package::load_module_tree_with_packages(&entry4, &idx).unwrap_err();
        assert!(matches!(err, PackageLoadError::Io(_)), "{err}");
    }
}

#[test]
fn rpi_edge_tokenize_skip_list_and_unclosed() {
    // Lone `(` then EOF → break without skip_list
    assert!(parse_rpi_exports("(").unwrap().is_empty());

    // Structured comment and unknown heads
    let exports = parse_rpi_exports("(// note) (mystery x) (fn dup (a) a) (fn dup (a) a)").unwrap();
    assert_eq!(exports, vec!["dup"]);

    // Unclosed list
    let err = parse_rpi_exports("(val x").unwrap_err();
    assert!(matches!(err, PackageLoadError::Interface(_)));

    // Unclosed string → tokenize Err mapped via L11
    let err = parse_rpi_exports("(val \"oops").unwrap_err();
    assert!(matches!(err, PackageLoadError::Interface(_)));

    // skip_list on non-`(` (via empty val name already covered); force expected `(`
    // by malformed nesting after successful parse of an opening.
    let err = parse_rpi_exports("(val x (()").unwrap_err();
    assert!(matches!(err, PackageLoadError::Interface(_)));
}

#[test]
fn rpxm_positional_name_entry_points_and_dep_edges() {
    // Positional DD-001 name when second token after package is `(`
    let m = parse_rpxm(
        r#"(package
  (name app)
  (version 1.0)
  (entry main.rpx))"#,
    )
    .unwrap();
    assert_eq!(m.name, "app");

    let m = parse_rpxm(
        r#"(package app
  format-version 1
  version "1"
  (entry-points cli)
  (public-modules main))"#,
    )
    .unwrap();
    assert_eq!(m.entry_points, vec!["cli".to_string()]);
    assert_eq!(m.entry, "cli.rpx");

    // Dependency missing alias content / unclosed + nested group depth
    let err = parse_rpxm(
        r#"(package app
  format-version 1
  version "1"
  (public-modules main)
  (dependencies
    ()))"#,
    )
    .unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));

    let err = parse_rpxm(
        r#"(package app
  format-version 1
  version "1"
  (public-modules main)
  (dependencies
    (g package graphics version "1" (nested (more)) path "../g")"#,
    )
    .unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));

    // Unclosed structured comment
    let err = parse_rpxm("(// never closed").unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));

    // Comment with nested parens and string
    let m = parse_rpxm(
        r#"(// outer (inner "str") more)
(package
  (name demo)
  (version 0.1.0)
  (entry main.rpx))"#,
    )
    .unwrap();
    assert_eq!(m.name, "demo");
}

#[test]
fn path_dep_without_package_field_uses_parsed_name() {
    let root = scratch();
    write_pkg(
        &root,
        "util",
        r#"(package util
  format-version 1
  version "1"
  (public-modules main))"#,
    );
    let consumer = root.join("app");
    fs::create_dir_all(&consumer).unwrap();
    fs::write(
        consumer.join("package.rpxm"),
        r#"(package app
  format-version 1
  version "1"
  (public-modules main)
  (dependencies
    (u version "1" path "../util")))"#,
    )
    .unwrap();
    let (idx, _) =
        LocalPackageIndex::discover_with_consumer(&[root.join("pkgs")], &consumer).unwrap();
    assert_eq!(idx.resolve_alias("u"), "util");
}

#[test]
fn remaining_load_error_branches_and_cycles() {
    // Corrupt path-dep manifest → parse Err inside register_path_dependencies.
    let root = scratch();
    let consumer = root.join("app");
    fs::create_dir_all(&consumer).unwrap();
    let dep = root.join("dep");
    fs::create_dir_all(&dep).unwrap();
    fs::write(dep.join("package.rpxm"), "(package").unwrap();
    fs::write(
        consumer.join("package.rpxm"),
        r#"(package app
  format-version 1
  version "1"
  (public-modules main)
  (dependencies
    (d package dep version "1" path "../dep")))"#,
    )
    .unwrap();
    let mut idx = LocalPackageIndex::discover(&[scratch()]).unwrap();
    let man = parse_rpxm(&fs::read_to_string(consumer.join("package.rpxm")).unwrap()).unwrap();
    let err = idx.register_path_dependencies(&consumer, &man).unwrap_err();
    assert!(matches!(err, PackageLoadError::Manifest(_)));

    // discover_with_consumer: discover Err (ambiguous search roots)
    let a = scratch();
    let b = scratch();
    write_pkg(
        &a,
        "x",
        r#"(package shared
  format-version 1
  version "1"
  (public-modules main))"#,
    );
    write_pkg(
        &b,
        "y",
        r#"(package shared
  format-version 1
  version "1"
  (public-modules main))"#,
    );
    let consumer2 = scratch().join("app");
    fs::create_dir_all(&consumer2).unwrap();
    fs::write(
        consumer2.join("package.rpxm"),
        r#"(package app
  format-version 1
  version "1"
  (public-modules main))"#,
    )
    .unwrap();
    let err = LocalPackageIndex::discover_with_consumer(&[a, b], &consumer2).unwrap_err();
    assert!(matches!(err, PackageLoadError::Ambiguous(_)));

    // discover_with_consumer: register Err (missing path dep)
    let consumer3 = scratch().join("app");
    fs::create_dir_all(&consumer3).unwrap();
    fs::write(
        consumer3.join("package.rpxm"),
        r#"(package app
  format-version 1
  version "1"
  (public-modules main)
  (dependencies
    (g package graphics version "1" path "../missing-graphics")))"#,
    )
    .unwrap();
    let err = LocalPackageIndex::discover_with_consumer(&[scratch()], &consumer3).unwrap_err();
    assert!(matches!(err, PackageLoadError::NotFound(_)));

    // Bad .rpi contents → parse_rpi_exports Err during resolve
    let root4 = scratch();
    write_pkg(
        &root4,
        "graphics",
        r#"(package graphics
  format-version 1
  version "0.1.0"
  source-root "src"
  interface-root "interface"
  (public-modules shapes))"#,
    );
    fs::create_dir_all(root4.join("graphics/src")).unwrap();
    fs::create_dir_all(root4.join("graphics/interface")).unwrap();
    fs::write(root4.join("graphics/src/shapes.rpx"), "(val circle 1)\n").unwrap();
    fs::write(root4.join("graphics/interface/shapes.rpi"), "(val").unwrap();
    let idx4 = LocalPackageIndex::discover(&[&root4]).unwrap();
    let err = idx4.resolve_import_detailed("graphics/shapes").unwrap_err();
    assert!(matches!(err, PackageLoadError::Interface(_)));

    // Cyclic imports hit `seen` continue
    let cyc = scratch();
    fs::write(cyc.join("a.rpx"), "(import b)\n(val main 1)\n").unwrap();
    fs::write(cyc.join("b.rpx"), "(import a)\n(val other 2)\n").unwrap();
    let idx5 = LocalPackageIndex::discover(&[scratch()]).unwrap();
    let units =
        reciplexa_package::load_module_tree_with_packages(cyc.join("a.rpx"), &idx5).unwrap();
    assert_eq!(units.len(), 2);

    // parse_imports Err on a non-entry unit
    let bad = scratch();
    fs::write(bad.join("entry.rpx"), "(import badmod)\n(val main 1)\n").unwrap();
    fs::write(bad.join("badmod.rpx"), "(import)\n").unwrap();
    let err = reciplexa_package::load_module_tree_with_packages(bad.join("entry.rpx"), &idx5)
        .unwrap_err();
    assert!(matches!(err, PackageLoadError::Module(_)));

    // elaborate_with_packages load Err
    let err = reciplexa_package::elaborate_with_packages(PathBuf::from("/no/entry.rpx"), &idx5)
        .unwrap_err();
    assert!(matches!(err, PackageLoadError::Io(_)));
}

#[test]
fn lockfile_from_graph_none_package_and_workspace_tokenize_err() {
    let m = PackageManifest {
        name: "app".into(),
        version: "1".into(),
        dependencies: vec![DependencySpec {
            name: "lib".into(),
            version_req: "1".into(),
            path: None,
            package: None,
        }],
        entry: "main.rpx".into(),
        ..Default::default()
    };
    let lf = Lockfile::from_graph(&[m]);
    assert_eq!(lf.packages[0].dependencies, vec!["lib".to_string()]);

    let err = parse_workspace_rpxm("(workspace (members \"unterminated)").unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));
}

#[test]
fn rpi_skip_list_errors_after_fn_comment_unknown() {
    let err = parse_rpi_exports("(fn foo (x").unwrap_err();
    assert!(matches!(err, PackageLoadError::Interface(_)));
    let err = parse_rpi_exports("(// unclosed").unwrap_err();
    assert!(matches!(err, PackageLoadError::Interface(_)));
    let err = parse_rpi_exports("(mystery").unwrap_err();
    assert!(matches!(err, PackageLoadError::Interface(_)));
}

#[test]
fn rpxm_entry_points_err_and_dependency_alias_missing() {
    let err = parse_rpxm(
        r#"(package app
  format-version 1
  version "1"
  (entry-points cli"#,
    )
    .unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));

    // EOF immediately after dependency `(` → missing alias
    let err = parse_rpxm(
        r#"(package app
  format-version 1
  version "1"
  (public-modules main)
  (dependencies
    ("#,
    )
    .unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)), "{err:?}");

    // Nested group that closes, then unclosed dep entry
    let err = parse_rpxm(
        r#"(package app
  format-version 1
  version "1"
  (public-modules main)
  (dependencies
    (g package graphics version "1" (a (b (c)))"#,
    )
    .unwrap_err();
    assert!(matches!(err, RpxmError::Syntax(_)));

    // Spaced structured comment `( // …)` hits comment-detector whitespace loop
    let m = parse_rpxm(
        r#"( // spaced comment (inner) )
(package
  (name demo)
  (version 0.1.0)
  (entry main.rpx))"#,
    )
    .unwrap();
    assert_eq!(m.name, "demo");

    // Bare unknown token inside dependency entry (`_ => i += 1`)
    let m = parse_rpxm(
        r#"(package app
  format-version 1
  version "1"
  (public-modules main)
  (dependencies
    (g package graphics version "1" ignored-token path "../g")))"#,
    )
    .unwrap();
    assert_eq!(m.dependencies[0].path.as_deref(), Some("../g"));
}

#[cfg(windows)]
#[test]
fn discover_read_dir_denied_errors() {
    let root = scratch();
    // Child package present so discovery would otherwise succeed.
    write_pkg(
        &root,
        "ok",
        r#"(package ok
  format-version 1
  version "1"
  (public-modules main))"#,
    );
    let user = std::env::var("USERNAME").unwrap_or_else(|_| "Everyone".into());
    let status = std::process::Command::new("icacls")
        .args([root.to_str().unwrap(), "/deny", &format!("{user}:(RD)")])
        .status()
        .expect("icacls");
    if !status.success() {
        // Environment may disallow ACL edits; skip quietly.
        return;
    }
    let err = LocalPackageIndex::discover(&[&root]);
    let _ = std::process::Command::new("icacls")
        .args([root.to_str().unwrap(), "/remove:d", &user])
        .status();
    if let Err(e) = err {
        assert!(matches!(e, PackageLoadError::Io(_)), "{e}");
    }
}
