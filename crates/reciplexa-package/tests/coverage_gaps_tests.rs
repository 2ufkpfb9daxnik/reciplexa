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
    let err = idx
        .register_path_dependencies(&consumer, &man)
        .unwrap_err();
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
    let err = idx
        .register_path_dependencies(&consumer, &man)
        .unwrap_err();
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
    let err = idx
        .register_path_dependencies(&consumer, &man)
        .unwrap_err();
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
