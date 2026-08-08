//! TEST-SYN-C004: reader/macro phase ordering smoke test.

use reciplexa_syntax::parse_source;
use reciplexa_test::{run_conformance, ConformanceCase};

#[test]
fn test_syn_c004_macro_forms_parse() {
    let case = ConformanceCase::new("TEST-SYN-C004", "SYN/MAC", "reader to macro phase ordering");
    run_conformance(&case, || {
        let src = "(define-macro m (lambda (x) x))";
        let parse = parse_source(src);
        assert!(parse.errors.is_empty(), "{:?}", parse.errors);
    });
}
