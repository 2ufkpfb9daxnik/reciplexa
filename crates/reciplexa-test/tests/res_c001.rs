//! TEST-RES-C001 / TEST-RES-001: lexical shadowing and unbound identifiers.

use reciplexa_bind::resolve_language_source;
use reciplexa_test::{run_conformance, ConformanceCase};

#[test]
fn test_res_c001_lexical_shadowing_and_unbound() {
    let case = ConformanceCase::new(
        "TEST-RES-C001",
        "RES-001",
        "lexical shadowing, fn params, unbound identifier",
    );
    run_conformance(&case, || {
        let shadow = resolve_language_source("(val x 1) (val main (let ((x 2)) x))");
        assert!(shadow.is_ok(), "{:?}", shadow.errors);

        let unbound = resolve_language_source("(val main y)");
        assert!(!unbound.is_ok());
        assert!(unbound
            .errors
            .iter()
            .any(|e| e.message.contains("unbound identifier `y`")));

        let fn_param = resolve_language_source("(val main ((fn (x) x) 1))");
        assert!(fn_param.is_ok(), "{:?}", fn_param.errors);
    });
}
