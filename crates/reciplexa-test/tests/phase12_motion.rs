//! Phase 12 motion IR conformance (TEST-IR-007 style).

use reciplexa_motion::*;
use reciplexa_test::{run_conformance, ConformanceCase};

#[test]
fn test_ir_007_animation_sample_determinism() {
    let case = ConformanceCase::new("TEST-IR-007", "Phase 12", "sample determinism");
    run_conformance(&case, || {
        let track = MotionTrack::keyframes(vec![
            Keyframe {
                at: TimeMs(0),
                value: 0.0,
                easing_to_next: Easing::Linear,
            },
            Keyframe {
                at: TimeMs(2000),
                value: 100.0,
                easing_to_next: Easing::Linear,
            },
        ]);
        let a = sample_f64(&track, TimeMs(1000), SampleMode::Preview).unwrap();
        let b = sample_f64(&track, TimeMs(1000), SampleMode::Preview).unwrap();
        assert!((a - b).abs() < 1e-12);
        assert!((a - 50.0).abs() < 1e-9);
    });
}

#[test]
fn strict_mode_rejects_keyframe_lerp() {
    let case = ConformanceCase::new("TEST-SLICE-012-STRICT", "Phase 12", "strict sampling");
    run_conformance(&case, || {
        let track = MotionTrack::keyframes(vec![
            Keyframe {
                at: TimeMs(0),
                value: 1.0,
                easing_to_next: Easing::EaseInOutCubic,
            },
            Keyframe {
                at: TimeMs(10),
                value: 2.0,
                easing_to_next: Easing::Linear,
            },
        ]);
        assert!(matches!(
            sample_f64(&track, TimeMs(5), SampleMode::Strict),
            Err(SampleError::StrictApproximation)
        ));
    });
}

#[test]
fn shared_node_identity_on_timeline_track() {
    let case = ConformanceCase::new("TEST-SLICE-012-ID", "Phase 12", "document identity");
    run_conformance(&case, || {
        let node = reciplexa_identity::document::StableNodeId::new(42);
        let tr = TimelineTrack {
            id: 1,
            node: Some(node),
            name: "opacity".into(),
            placement: TemporalPlacement::span(TimeMs(0), TimeMs(1000)),
            track: MotionTrack::constant(1.0),
        };
        assert_eq!(tr.node, Some(node));
    });
}
