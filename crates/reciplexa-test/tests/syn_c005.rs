//! TEST-SYN-C005: Unicode identifiers and deep nesting.

use reciplexa_syntax::parse_source;
use reciplexa_test::{run_conformance, ConformanceCase};

#[test]
fn test_syn_c005_unicode_identifier() {
    let case = ConformanceCase::new("TEST-SYN-C005", "LEX", "Unicode identifier tokenization");
    run_conformance(&case, || {
        let src = "(let 変数 1 変数)";
        let parse = parse_source(src);
        assert!(parse.errors.is_empty(), "{:?}", parse.errors);
    });
}

#[test]
fn test_syn_c005_deep_nesting() {
    let case = ConformanceCase::new("TEST-SYN-C005", "LEX", "deep nest fuzz");
    run_conformance(&case, || {
        let src = "((((1)))))";
        let parse = parse_source(src);
        assert!(!parse.errors.is_empty());
    });
}
