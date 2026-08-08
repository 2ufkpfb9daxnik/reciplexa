//! TEST-SYN-C007: multiple independent diagnostics in one parse.

use reciplexa_syntax::parse_source;
use reciplexa_test::{run_conformance, ConformanceCase};

#[test]
fn test_syn_c007_multiple_parse_diagnostics() {
    let case = ConformanceCase::new(
        "TEST-SYN-C007",
        "LEX/SYN",
        "multiple diagnostics collected",
    );
    run_conformance(&case, || {
        // Unexpected close, then another malformed form.
        let parse = parse_source(") (unclosed");
        assert!(
            parse.errors.len() >= 2,
            "expected >=2 errors, got {}",
            parse.errors.len()
        );
        assert!(parse.has_errors());
        assert!(parse
            .root
            .descendants()
            .any(|n| n.kind() == reciplexa_syntax::SyntaxKind::ErrorNode));
    });
}
