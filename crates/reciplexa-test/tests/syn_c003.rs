//! TEST-SYN-C003: code/doc nesting parses without errors.

use reciplexa_syntax::parse_source;
use reciplexa_test::{run_conformance, ConformanceCase};

#[test]
fn test_syn_c003_code_doc_nesting() {
    let case = ConformanceCase::new("TEST-SYN-C003", "SYN", "code/doc nesting and escapes");
    run_conformance(&case, || {
        let src = "(markup Hello @em{world}.)";
        let parse = parse_source(src);
        assert!(parse.errors.is_empty(), "{:?}", parse.errors);
    });
}
