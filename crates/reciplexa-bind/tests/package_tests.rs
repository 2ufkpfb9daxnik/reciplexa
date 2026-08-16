//! Integration tests moved from src/package.rs for region coverage.

use reciplexa_bind::package::*;
use reciplexa_identity::package::PackageInstanceId;

#[test]
fn resolves_multiple_units() {
    let units: [(&str, &str); 2] = [
        ("main", "(val main 1)"),
        ("lib", "(val helper 2)"),
    ];
    let refs: Vec<(String, &str)> = units.iter().map(|(n, s)| (n.to_string(), *s)).collect();
    let pkg = resolve_package(PackageInstanceId::new(1), &refs);
    assert_eq!(pkg.skeleton.units.len(), 2);
    assert!(pkg.modules.values().all(|m| m.is_ok()));
}
