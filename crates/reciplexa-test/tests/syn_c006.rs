//! TEST-SYN-C006: no infix precedence ambiguity in s-expressions.

use reciplexa_syntax::parse_source;
use reciplexa_test::{run_conformance, ConformanceCase};

#[test]
fn test_syn_c006_prefix_only_expressions() {
    let case = ConformanceCase::new("TEST-SYN-C006", "SYN", "no infix precedence ambiguity");
    run_conformance(&case, || {
        let src = "(+ 1 (* 2 3))";
        let parse = parse_source(src);
        assert!(parse.errors.is_empty(), "{:?}", parse.errors);
    });
}
