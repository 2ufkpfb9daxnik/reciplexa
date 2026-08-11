//! Tip remaining Err arms on top-level perform/handle walkers.

use reciplexa_effect::{collect_performs, run_source_effects, TestHandler};

#[test]
fn collect_top_level_perform_and_handle_error_arms() {
    // Malformed top-level perform → Err through L140 `?`
    assert!(collect_performs("(perform log)").is_err());
    // Malformed handle body → Err through L142 `?`
    assert!(collect_performs("(handle log (perform log))").is_err());
}

#[test]
fn run_source_effects_top_level_error_arms() {
    let mut h = TestHandler::default();
    // Top-level perform/handle Err through L162 `?`
    assert!(run_source_effects(&mut h, "(perform log)").is_err());
    assert!(run_source_effects(&mut h, "(handle log (perform log))").is_err());
}
