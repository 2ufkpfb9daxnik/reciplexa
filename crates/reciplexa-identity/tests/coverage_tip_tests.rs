//! Tip identity ID/provenance Display and accessor coverage.

use reciplexa_identity::package::{DefinitionId, FunctorId, SignatureId};
use reciplexa_identity::provenance::ProvenanceKind;

#[test]
fn package_ids_accessors_and_display() {
    let d = DefinitionId::new(7);
    assert_eq!(d.get(), 7);
    assert!(d.is_valid());
    assert!(!DefinitionId::INVALID.is_valid());
    assert_eq!(d.to_string(), "definition:7");

    let s = SignatureId::new(3);
    assert_eq!(s.get(), 3);
    assert!(s.is_valid());
    assert_eq!(s.to_string(), "signature:3");

    let f = FunctorId::new(9);
    assert_eq!(f.get(), 9);
    assert!(f.is_valid());
    assert_eq!(f.to_string(), "functor:9");
}

#[test]
fn provenance_kind_all_display_variants() {
    assert_eq!(ProvenanceKind::UserCreated.to_string(), "user-created");
    assert_eq!(
        ProvenanceKind::SourceGenerated.to_string(),
        "source-generated"
    );
    assert_eq!(
        ProvenanceKind::MacroGenerated.to_string(),
        "macro-generated"
    );
    assert_eq!(ProvenanceKind::Imported.to_string(), "imported");
    assert_eq!(ProvenanceKind::Copied.to_string(), "copied");
    assert_eq!(ProvenanceKind::Derived.to_string(), "derived");
}
