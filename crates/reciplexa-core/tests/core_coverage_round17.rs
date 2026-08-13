//! Round-17 core: residual elaborate/check clusters (quarantine, data rec, comments).

use reciplexa_core::check::typecheck_language_source;
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};

#[test]
fn elaborate_round17_quarantine_comments_and_data_rec() {
    let cases = [
        "(// structured top comment)\n(val main 1)",
        "(doc \"orphan\")\n(val main 1)",
        "(rec (not-data))\n(val main 1)",
        "(rec (data box ((a type)) (mk a)) (oops))\n(val main 1)",
        "(type-alias t int)\n(val main 1)",
        "(type t int)\n(type-alias u t)\n(val main 1)",
        "(data pair (mk a b))\n(val main (mk 1 2))",
        "(rec (data nat (zero) (succ n)))\n(val main zero)",
    ];
    for src in cases {
        let _ = elaborate_source(src);
        let _ = elaborate_with_data(src);
        let _ = typecheck_language_source(src);
    }
}

#[test]
fn elaborate_round17_top_token_and_unsupported_form() {
    let cases = ["1\n(val main 1)", "(val main 1) extra"];
    for src in cases {
        let _ = elaborate_source(src);
    }
}
