//! TEST-SYN-C001: valid parse/unparse byte round-trip.

use reciplexa_syntax::{parse_source, unparse};
use reciplexa_test::{run_conformance, ConformanceCase};

#[test]
fn test_syn_c001_parse_unparse_roundtrip() {
    let case = ConformanceCase::new(
        "TEST-SYN-C001",
        "LEX/SYN",
        "valid parse/unparse byte round-trip",
    );
    run_conformance(&case, || {
        let src = "(page a4 (circle 10 20 5))";
        let parse = parse_source(src);
        assert!(parse.errors.is_empty(), "{:?}", parse.errors);
        let round = unparse(&parse.root);
        let reparsed = parse_source(&round);
        assert!(reparsed.errors.is_empty(), "{:?}", reparsed.errors);
    });
}
