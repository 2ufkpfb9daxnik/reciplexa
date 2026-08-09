//! Phase 12 multi-backend conformance: SVG / Raster / Video + explicit losses.

use reciplexa_backend::{
    export_scene_to_raster, export_scene_to_svg, BackendCapability, LossDisposition,
    OutputLossKind, OutputProfile, ProfileKind,
};
use reciplexa_motion::*;
use reciplexa_raster::RasterOptions;
use reciplexa_scene::{Circle, Color, Document, Page, PaperSize, Shape, Text};
use reciplexa_test::{run_conformance, ConformanceCase};
use reciplexa_video::{sample_timeline_sequence, VideoOptions, VideoProfile};

fn tiny_doc_with_text() -> Document {
    Document::single_page(Page {
        paper: PaperSize {
            width_mm: 20.0,
            height_mm: 20.0,
        },
        shapes: vec![
            Shape::Circle(Circle {
                x_mm: 10.0,
                y_mm: 10.0,
                radius_mm: 4.0,
                fill: Color::BLUE,
            }),
            Shape::Text(Text {
                x_mm: 1.0,
                y_mm: 1.0,
                size_mm: 3.0,
                width_mm: None,
                height_mm: None,
                content: "hi".into(),
                fill: Color::BLACK,
            }),
        ],
    })
}

#[test]
fn phase12_svg_and_raster_share_document_identity_path() {
    let case = ConformanceCase::new("TEST-SLICE-012-MULTI", "Phase 12", "multi-backend");
    run_conformance(&case, || {
        let doc = tiny_doc_with_text();
        let svg = export_scene_to_svg(
            &doc,
            &BackendCapability::svg_default(),
            &OutputProfile::svg_default(),
        )
        .unwrap();
        assert!(svg.svg.contains("rpx-"));
        let raster = export_scene_to_raster(
            &doc,
            &BackendCapability::raster_default(),
            &OutputProfile::preview_raster(),
            0,
        )
        .unwrap();
        assert!(raster.png.starts_with(&[0x89, b'P', b'N', b'G']));
        assert_eq!(raster.losses.profile, ProfileKind::Preview);
        assert!(raster
            .losses
            .losses
            .iter()
            .any(|l| l.kind == OutputLossKind::SemanticText));
    });
}

#[test]
fn phase12_preview_vs_final_raster_losses_never_silent() {
    let case = ConformanceCase::new("TEST-SLICE-012-LOSS", "Phase 12", "profile losses");
    run_conformance(&case, || {
        let doc = tiny_doc_with_text();
        let preview = export_scene_to_raster(
            &doc,
            &BackendCapability::raster_default(),
            &OutputProfile::preview_raster(),
            0,
        )
        .unwrap();
        let final_a = export_scene_to_raster(
            &doc,
            &BackendCapability::raster_default(),
            &OutputProfile::final_raster(),
            0,
        )
        .unwrap();
        assert!(final_a.width > preview.width);
        assert_ne!(preview.losses.profile, final_a.losses.profile);
        let p = preview
            .losses
            .losses
            .iter()
            .find(|l| l.kind == OutputLossKind::SemanticText)
            .unwrap();
        let f = final_a
            .losses
            .losses
            .iter()
            .find(|l| l.kind == OutputLossKind::SemanticText)
            .unwrap();
        assert_eq!(p.disposition, LossDisposition::Report);
        assert_eq!(f.disposition, LossDisposition::RequireExplicitApproval);
    });
}

#[test]
fn phase12_motion_preview_final_sync_divergence() {
    let case = ConformanceCase::new("TEST-SLICE-012-SYNC", "Phase 12", "preview sync");
    run_conformance(&case, || {
        let mut tl = MotionTimeline {
            playhead: TimeMs(250),
            duration: DurationMs(1000),
            tracks: vec![TimelineTrack {
                id: 3,
                node: Some(reciplexa_identity::document::StableNodeId::new(9)),
                name: "x".into(),
                placement: TemporalPlacement::span(TimeMs(0), TimeMs(1000)),
                track: MotionTrack::keyframes(vec![
                    Keyframe {
                        at: TimeMs(0),
                        value: 0.0,
                        easing_to_next: Easing::Linear,
                    },
                    Keyframe {
                        at: TimeMs(1000),
                        value: 10.0,
                        easing_to_next: Easing::Linear,
                    },
                ]),
            }],
            playing: false,
        };
        let diff = compare_preview_final(&tl);
        assert!(diff.divergent_track_ids.contains(&3));
        tl.seek(TimeMs(0));
        let aligned = compare_preview_final(&tl);
        assert!(aligned.divergent_track_ids.is_empty());
    });
}

#[test]
fn phase12_video_samples_with_profile_losses() {
    let case = ConformanceCase::new("TEST-INT-007", "Phase 12", "video sample");
    run_conformance(&case, || {
        let tl = MotionTimeline {
            playhead: TimeMs::ZERO,
            duration: DurationMs(500),
            tracks: vec![TimelineTrack {
                id: 1,
                node: None,
                name: "a".into(),
                placement: TemporalPlacement::span(TimeMs(0), TimeMs(500)),
                track: MotionTrack::keyframes(vec![
                    Keyframe {
                        at: TimeMs(0),
                        value: 0.0,
                        easing_to_next: Easing::Linear,
                    },
                    Keyframe {
                        at: TimeMs(500),
                        value: 1.0,
                        easing_to_next: Easing::Linear,
                    },
                ]),
            }],
            playing: false,
        };
        let preview = sample_timeline_sequence(&tl, &VideoOptions::preview()).unwrap();
        let final_s = sample_timeline_sequence(&tl, &VideoOptions::final_out()).unwrap();
        assert_eq!(preview.profile, VideoProfile::Preview);
        assert_eq!(final_s.profile, VideoProfile::Final);
        assert!(final_s.fps > preview.fps);
        assert!(!final_s.losses.is_empty());
        let _ = RasterOptions::default();
    });
}
