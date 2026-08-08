//! TEST-SYN-C008: incremental reparse preserves unchanged subtree identity.

use reciplexa_syntax::{build_identity_map, parse_source, preserve_identity_on_reparse};
use reciplexa_test::{run_conformance, ConformanceCase};

#[test]
fn test_syn_c008_identity_preserved_after_small_edit() {
    let case = ConformanceCase::new(
        "TEST-SYN-C008",
        "LEX/SYN",
        "unchanged subtree identity preserved",
    );
    run_conformance(&case, || {
        let before = "(page a4 (rect 10 20 30 40) (circle 5 5 2))";
        let after = "(page a4 (rect 10 20 50 40) (circle 5 5 2))";
        let p1 = parse_source(before);
        let p2 = parse_source(after);
        let map1 = build_identity_map(&p1.root);
        let map2 = preserve_identity_on_reparse(&map1, &p1.root, &p2.root);

        let circle1 = p1
            .root
            .descendants()
            .find(|n| n.text().to_string().contains("circle"))
            .expect("circle form");
        let circle2 = p2
            .root
            .descendants()
            .find(|n| n.text().to_string().contains("circle"))
            .expect("circle form");
        assert_eq!(
            map1.get(circle1.text_range()),
            map2.get(circle2.text_range()),
            "unchanged circle subtree should keep syntax id"
        );
    });
}
