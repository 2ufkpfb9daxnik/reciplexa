//! ResourceQuota tests.

use reciplexa_harden::{QuotaExceeded, ResourceKind, ResourceQuota};

#[test]
fn charge_and_release_memory() {
    let mut q = ResourceQuota::new(100, 10, 1000, 1000);
    assert!(q.charge(ResourceKind::MemoryBytes, 40).is_ok());
    assert_eq!(q.used(ResourceKind::MemoryBytes), 40);
    q.release(ResourceKind::MemoryBytes, 10);
    assert_eq!(q.used(ResourceKind::MemoryBytes), 30);
}

#[test]
fn exceeds_open_files() {
    let mut q = ResourceQuota::new(1000, 2, 1000, 1000);
    assert!(q.charge(ResourceKind::OpenFiles, 2).is_ok());
    let err: QuotaExceeded = q.charge(ResourceKind::OpenFiles, 1).unwrap_err();
    assert_eq!(err.kind, ResourceKind::OpenFiles);
}

#[test]
fn wall_and_cpu_limits() {
    let mut q = ResourceQuota::default();
    assert!(q.charge(ResourceKind::WallTimeMs, 1).is_ok());
    assert!(q.charge(ResourceKind::CpuTimeMs, 1).is_ok());
    let mut tight = ResourceQuota::new(1, 1, 5, 5);
    assert_eq!(
        tight.charge(ResourceKind::WallTimeMs, 6).unwrap_err().kind,
        ResourceKind::WallTimeMs
    );
    assert_eq!(
        tight.charge(ResourceKind::CpuTimeMs, 6).unwrap_err().kind,
        ResourceKind::CpuTimeMs
    );
}
