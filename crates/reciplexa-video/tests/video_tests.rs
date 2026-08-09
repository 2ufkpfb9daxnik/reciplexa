use reciplexa_motion::*;
use reciplexa_raster::RasterOptions;
use reciplexa_scene::{Circle, Color, Document, Page, PaperSize, Shape, Text};
use reciplexa_video::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn simple_timeline() -> MotionTimeline {
    let track = MotionTrack::keyframes(vec![
        Keyframe {
            at: TimeMs(0),
            value: 0.0,
            easing_to_next: Easing::Linear,
        },
        Keyframe {
            at: TimeMs(1000),
            value: 100.0,
            easing_to_next: Easing::Linear,
        },
    ]);
    MotionTimeline {
        playhead: TimeMs::ZERO,
        duration: DurationMs(1000),
        tracks: vec![TimelineTrack {
            id: 1,
            node: None,
            name: "x".into(),
            placement: TemporalPlacement::span(TimeMs(0), TimeMs(1000)),
            track,
        }],
        playing: false,
    }
}

#[test]
fn preview_samples_interpolated_midpoint() {
    let tl = simple_timeline();
    let seq = sample_timeline_sequence(&tl, &VideoOptions::preview()).unwrap();
    assert_eq!(seq.profile, VideoProfile::Preview);
    assert!(seq.frames.len() >= 12);
    let mid = seq
        .frames
        .iter()
        .find(|f| f.time.0 == 500)
        .or_else(|| seq.frames.iter().find(|f| (400..600).contains(&f.time.0)))
        .expect("mid frame");
    let v = mid.tracks[0].value;
    assert!(v > 30.0 && v < 70.0, "got {v}");
}

#[test]
fn final_reports_strict_approximation_losses() {
    let tl = simple_timeline();
    let seq = sample_timeline_sequence(&tl, &VideoOptions::final_out()).unwrap();
    assert_eq!(seq.profile, VideoProfile::Final);
    assert!(seq.fps > VideoOptions::preview().fps);
    assert!(
        seq.losses
            .iter()
            .any(|l| l.kind == VideoLossKind::StrictApproximationSkipped),
        "Final must report strict interpolation skips, got {:?}",
        seq.losses
    );
}

#[test]
fn preview_vs_final_profiles_differ() {
    assert_ne!(
        VideoProfile::Preview.sample_mode(),
        VideoProfile::Final.sample_mode()
    );
    assert!(VideoProfile::Final.default_fps() > VideoProfile::Preview.default_fps());
}

#[test]
fn rejects_bad_fps() {
    let tl = simple_timeline();
    let mut opts = VideoOptions::preview();
    opts.fps = f64::NAN;
    assert_eq!(
        sample_timeline_sequence(&tl, &opts).unwrap_err(),
        VideoError::NonFiniteFps
    );
    opts.fps = 0.0;
    assert_eq!(
        sample_timeline_sequence(&tl, &opts).unwrap_err(),
        VideoError::NonPositiveFps
    );
}

#[test]
fn rejects_empty_timeline() {
    let tl = MotionTimeline::default();
    assert_eq!(
        sample_timeline_sequence(&tl, &VideoOptions::preview()).unwrap_err(),
        VideoError::EmptyTimeline
    );
}

#[test]
fn writes_png_sequence_to_temp_dir() {
    let tl = simple_timeline();
    let doc = Document::single_page(Page {
        paper: PaperSize {
            width_mm: 5.0,
            height_mm: 5.0,
        },
        shapes: vec![Shape::Circle(Circle {
            x_mm: 2.5,
            y_mm: 2.5,
            radius_mm: 1.0,
            fill: Color::RED,
        })],
    });
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("reciplexa-video-{stamp}"));
    let (seq, paths) = export_png_sequence(
        &tl,
        &doc,
        0,
        &VideoOptions {
            profile: VideoProfile::Preview,
            fps: 4.0,
        },
        &RasterOptions::default(),
        &dir,
    )
    .unwrap();
    assert!(!seq.frames.is_empty());
    assert_eq!(paths.len(), seq.frames.len());
    for p in &paths {
        let bytes = std::fs::read(p).unwrap();
        assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G']));
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn zero_duration_with_tracks_yields_one_frame() {
    let track = MotionTrack::constant(1.0);
    let tl = MotionTimeline {
        playhead: TimeMs::ZERO,
        duration: DurationMs(0),
        tracks: vec![TimelineTrack {
            id: 9,
            node: None,
            name: "c".into(),
            placement: TemporalPlacement::span(TimeMs(0), TimeMs(1)),
            track,
        }],
        playing: false,
    };
    let seq = sample_timeline_sequence(&tl, &VideoOptions::preview()).unwrap();
    assert_eq!(seq.frames.len(), 1);
    assert_eq!(seq.frames[0].tracks[0].value, 1.0);
}

#[test]
fn transparent_policy_records_video_loss() {
    let mut placement = TemporalPlacement::span(TimeMs(100), TimeMs(200));
    placement.range_policy = RangePolicy::Transparent;
    let tl = MotionTimeline {
        playhead: TimeMs::ZERO,
        duration: DurationMs(50),
        tracks: vec![TimelineTrack {
            id: 7,
            node: None,
            name: "out".into(),
            placement,
            track: MotionTrack::constant(1.0),
        }],
        playing: false,
    };
    let seq = sample_timeline_sequence(
        &tl,
        &VideoOptions {
            profile: VideoProfile::Preview,
            fps: 10.0,
        },
    )
    .unwrap();
    assert!(seq
        .losses
        .iter()
        .any(|l| l.kind == VideoLossKind::TrackTransparent && l.track_id == Some(7)));
}

#[test]
fn empty_range_policy_yields_no_sample() {
    let mut placement = TemporalPlacement::span(TimeMs(0), TimeMs(1));
    placement.parent_start = TimeMs(5);
    placement.parent_end = TimeMs(5);
    placement.range_policy = RangePolicy::Failure;
    let tl = MotionTimeline {
        playhead: TimeMs::ZERO,
        duration: DurationMs(20),
        tracks: vec![TimelineTrack {
            id: 8,
            node: None,
            name: "empty".into(),
            placement,
            track: MotionTrack::constant(1.0),
        }],
        playing: false,
    };
    let seq = sample_timeline_sequence(
        &tl,
        &VideoOptions {
            profile: VideoProfile::Preview,
            fps: 5.0,
        },
    )
    .unwrap();
    assert!(seq.frames.iter().all(|f| f.tracks.is_empty()));
}

#[test]
fn nonfinite_sample_records_track_failure_loss() {
    let tl = MotionTimeline {
        playhead: TimeMs::ZERO,
        duration: DurationMs(10),
        tracks: vec![TimelineTrack {
            id: 11,
            node: None,
            name: "nan".into(),
            placement: TemporalPlacement::span(TimeMs(0), TimeMs(10)),
            track: MotionTrack::constant(f64::NAN),
        }],
        playing: false,
    };
    let seq = sample_timeline_sequence(
        &tl,
        &VideoOptions {
            profile: VideoProfile::Preview,
            fps: 10.0,
        },
    )
    .unwrap();
    assert!(seq.losses.iter().any(|l| l.detail.contains("failed")));
}

#[test]
fn rasterize_sequence_merges_raster_losses() {
    let tl = simple_timeline();
    let sampled = sample_timeline_sequence(
        &tl,
        &VideoOptions {
            profile: VideoProfile::Preview,
            fps: 2.0,
        },
    )
    .unwrap();
    let doc = Document::single_page(Page {
        paper: PaperSize {
            width_mm: 4.0,
            height_mm: 4.0,
        },
        shapes: vec![Shape::Text(Text {
            x_mm: 0.5,
            y_mm: 0.5,
            size_mm: 1.0,
            width_mm: None,
            height_mm: None,
            content: "x".into(),
            fill: Color::BLACK,
        })],
    });
    let seq = rasterize_sampled_sequence(&doc, &sampled, 0, &RasterOptions::default()).unwrap();
    assert!(!seq.frames.is_empty());
    assert!(seq.losses.iter().any(|l| l.detail.contains("raster loss")));
}

#[test]
fn write_png_sequence_encode_error() {
    let tl = simple_timeline();
    let sampled = sample_timeline_sequence(
        &tl,
        &VideoOptions {
            profile: VideoProfile::Preview,
            fps: 2.0,
        },
    )
    .unwrap();
    let doc = Document::single_page(Page {
        paper: PaperSize {
            width_mm: 3.0,
            height_mm: 3.0,
        },
        shapes: vec![],
    });
    let mut seq = rasterize_sampled_sequence(&doc, &sampled, 0, &RasterOptions::default()).unwrap();
    seq.frames[0].frame.rgb.clear();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("reciplexa-video-enc-{stamp}"));
    let err = write_png_sequence(&seq, &dir).unwrap_err();
    assert!(matches!(err, WriteError::Encode(_)));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn export_png_sequence_maps_pipeline_errors() {
    let empty = MotionTimeline::default();
    let doc = Document::single_page(Page {
        paper: PaperSize {
            width_mm: 2.0,
            height_mm: 2.0,
        },
        shapes: vec![],
    });
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("reciplexa-video-pipe-{stamp}"));
    let err = export_png_sequence(
        &empty,
        &doc,
        0,
        &VideoOptions::preview(),
        &RasterOptions::default(),
        &dir,
    )
    .unwrap_err();
    assert!(matches!(err, VideoPipelineError::Sample(_)));

    let tl = simple_timeline();
    let err = export_png_sequence(
        &tl,
        &doc,
        99,
        &VideoOptions {
            profile: VideoProfile::Preview,
            fps: 2.0,
        },
        &RasterOptions::default(),
        &dir,
    )
    .unwrap_err();
    assert!(matches!(err, VideoPipelineError::Raster(_)));

    let err = export_png_sequence(
        &tl,
        &doc,
        0,
        &VideoOptions {
            profile: VideoProfile::Preview,
            fps: 2.0,
        },
        &RasterOptions::default(),
        std::path::Path::new("Z:\\reciplexa-missing-drive\\x"),
    )
    .unwrap_err();
    assert!(matches!(err, VideoPipelineError::Write(_)));
}

#[test]
fn write_fails_when_frame_path_is_directory() {
    let tl = simple_timeline();
    let sampled = sample_timeline_sequence(
        &tl,
        &VideoOptions {
            profile: VideoProfile::Preview,
            fps: 2.0,
        },
    )
    .unwrap();
    let doc = Document::single_page(Page {
        paper: PaperSize {
            width_mm: 2.0,
            height_mm: 2.0,
        },
        shapes: vec![],
    });
    let seq = rasterize_sampled_sequence(&doc, &sampled, 0, &RasterOptions::default()).unwrap();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("reciplexa-video-clash-{stamp}"));
    std::fs::create_dir_all(dir.join("frame_00000.png")).unwrap();
    let err = write_png_sequence(&seq, &dir).unwrap_err();
    assert!(matches!(err, WriteError::Io(_)));
    let _ = std::fs::remove_dir_all(&dir);
}
