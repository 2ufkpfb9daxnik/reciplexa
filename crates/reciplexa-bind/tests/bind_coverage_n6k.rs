//! N6 tip k: module.rs residual tree / OS read Err via exclusive lock (no FS seam).

use std::fs::{self, File, OpenOptions};
use std::io::Write;

use reciplexa_bind::module::{
    elaborate_module_tree, elaborate_units, load_module_tree, parse_imports, ModuleSkeleton,
    ModuleUnit,
};
use reciplexa_identity::package::{ModuleId, PackageInstanceId};
use reciplexa_source::resource::SourceResourceId;

#[cfg(windows)]
fn exclusive_lock(path: &std::path::Path) -> File {
    use std::os::windows::fs::OpenOptionsExt;
    let mut f = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .share_mode(0) // FILE_SHARE_NONE → concurrent read fails
        .open(path)
        .expect("exclusive lock");
    f.write_all(b"(val main 1)\n").unwrap();
    f.flush().unwrap();
    f
}

#[test]
fn module_n6k_skeleton_and_missing_import_file() {
    let mut sk = ModuleSkeleton::new(PackageInstanceId::new(1));
    let mid = ModuleId::new(1);
    sk.add_unit(ModuleUnit {
        module_id: mid,
        source_resource_id: SourceResourceId::new(1),
        name: "main".into(),
    });
    assert!(sk.find(mid).is_some());
    assert!(sk.find(ModuleId::new(99)).is_none());
    assert_eq!(sk.entry, Some(mid));

    let dir = std::env::temp_dir().join("rpx_bind_n6k_miss");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("main.rpx"), "(import missinglib)\n(val main 1)\n").unwrap();
    let err = load_module_tree(dir.join("main.rpx")).expect_err("missing sibling");
    assert!(
        err.message.contains("failed to read module"),
        "{}",
        err.message
    );
    let _ = elaborate_module_tree(dir.join("main.rpx"));

    for src in [
        "(val main 1)\n",
        " \n(val main 1)",
        "(1 2 3)\n(val main 1)",
        "(import lib)\n(val main 1)",
    ] {
        let _ = parse_imports(src);
    }
    let _ = elaborate_units(&[("main", "(val main 1)")]);
}

#[cfg(windows)]
#[test]
fn module_n6k_exclusive_lock_read_err_arms() {
    let dir = std::env::temp_dir().join("rpx_bind_n6k_lock");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    // Entry-file read map_err while exclusively locked.
    let entry = dir.join("locked.rpx");
    let _guard = exclusive_lock(&entry);
    let err = load_module_tree(&entry).expect_err("locked entry");
    assert!(
        err.message.contains("failed to read module") || err.message.contains("failed to read"),
        "{}",
        err.message
    );
    let err = elaborate_module_tree(&entry).expect_err("locked elaborate");
    assert!(!err.message.is_empty());

    // Directory unit read map_err: lock one .rpx, leave another readable so
    // read_dir succeeds then per-file read fails.
    let dir2 = std::env::temp_dir().join("rpx_bind_n6k_lock_dir");
    let _ = fs::remove_dir_all(&dir2);
    fs::create_dir_all(&dir2).unwrap();
    fs::write(dir2.join("ok.rpx"), "(val a 1)\n").unwrap();
    let locked = dir2.join("locked.rpx");
    let _guard2 = exclusive_lock(&locked);
    let err = load_module_tree(&dir2).expect_err("locked dir unit");
    assert!(err.message.contains("failed to read"), "{}", err.message);
}

#[cfg(windows)]
#[test]
fn module_n6k_non_utf8_stem_err_arms() {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    let dir = std::env::temp_dir().join("rpx_bind_n6k_utf8");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    // Entry file with non-UTF-8 stem → ok_or_else UTF-8 stem Err.
    let mut name = OsString::from_wide(&[0xD800]);
    name.push(".rpx");
    let entry = dir.join(&name);
    fs::write(&entry, "(val main 1)\n").unwrap();
    let err = load_module_tree(&entry).expect_err("non-utf8 stem");
    assert!(
        err.message.contains("UTF-8") || err.message.contains("stem"),
        "{}",
        err.message
    );

    // Directory scan: non-UTF-8 .rpx stem → non-UTF-8 module path Err.
    let dir2 = std::env::temp_dir().join("rpx_bind_n6k_utf8_dir");
    let _ = fs::remove_dir_all(&dir2);
    fs::create_dir_all(&dir2).unwrap();
    fs::write(dir2.join("ok.rpx"), "(val a 1)\n").unwrap();
    let bad = dir2.join(&name);
    fs::write(&bad, "(val b 1)\n").unwrap();
    let err = load_module_tree(&dir2).expect_err("non-utf8 dir unit");
    assert!(
        err.message.contains("non-UTF-8") || err.message.contains("UTF-8"),
        "{}",
        err.message
    );
}

#[cfg(windows)]
#[test]
fn module_n6k_exclusive_lock_directory_read_dir_err() {
    use std::os::windows::fs::OpenOptionsExt;

    let dir = std::env::temp_dir().join("rpx_bind_n6k_lock_readdir");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("a.rpx"), "(val main 1)\n").unwrap();

    // Lock the directory itself (backup semantics + no share) so read_dir fails.
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    let _dir_guard = OpenOptions::new()
        .read(true)
        .share_mode(0)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(&dir)
        .expect("lock directory");

    let err = load_module_tree(&dir).expect_err("locked read_dir");
    assert!(
        err.message.contains("failed to read module directory")
            || err.message.contains("failed to read"),
        "{}",
        err.message
    );
    let _ = elaborate_module_tree(&dir);
}
