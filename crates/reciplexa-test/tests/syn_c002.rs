//! TEST-SYN-C002: malformed input yields recoverable diagnostics.

use reciplexa_syntax::parse_source;
use reciplexa_test::{run_conformance, ConformanceCase};

#[test]
fn test_syn_c002_malformed_input_reports_errors() {
    let case = ConformanceCase::new("TEST-SYN-C002", "LEX/SYN", "malformed input recovery");
    run_conformance(&case, || {
        let parse = parse_source("(unclosed");
        assert!(!parse.errors.is_empty());
        assert!(parse.has_errors());
    });
}
