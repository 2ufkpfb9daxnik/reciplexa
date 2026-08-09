//! Capability / privacy audit tests.

use reciplexa_harden::{
    AuditFinding, AuditKind, AuditSeverity, CapabilityAudit, PrivacyAudit, PrivacyLabel,
};

#[test]
fn capability_missing_and_extra() {
    let a = CapabilityAudit::new(
        vec!["fs.read".into(), "net.none".into()],
        vec!["fs.read".into(), "gpu".into()],
    );
    let findings = a.findings();
    assert!(findings.iter().any(|f| f.code == "CAP001"));
    assert!(findings.iter().any(|f| f.code == "CAP002"));
    assert!(!a.is_clean());
    assert_eq!(findings[0].kind, AuditKind::Capability);
}

#[test]
fn clean_when_exact_match() {
    let a = CapabilityAudit::new(vec!["a".into()], vec!["a".into()]);
    assert!(a.is_clean());
    assert!(a.findings().is_empty());
}

#[test]
fn privacy_flags_secrets_and_unknown() {
    let p = PrivacyAudit::new(vec![
        ("title".into(), PrivacyLabel::Public),
        ("token".into(), PrivacyLabel::Secret),
        ("maybe".into(), PrivacyLabel::Unknown),
        ("note".into(), PrivacyLabel::Internal),
    ]);
    assert!(p.has_secrets());
    let f = p.findings();
    assert!(f
        .iter()
        .any(|x| x.code == "PRIV001" && x.severity == AuditSeverity::Error));
    assert!(f.iter().any(|x| x.code == "PRIV002"));
}

#[test]
fn audit_finding_constructor_and_repro_kind() {
    let f = AuditFinding::new(
        AuditKind::ReproducibleBuild,
        AuditSeverity::Info,
        "REPRO001",
        "inputs pinned",
    );
    assert_eq!(f.kind, AuditKind::ReproducibleBuild);
    assert_eq!(f.severity, AuditSeverity::Info);
    let empty = CapabilityAudit::default();
    assert!(empty.is_clean());
    let priv_empty = PrivacyAudit::default();
    assert!(!priv_empty.has_secrets());
}
