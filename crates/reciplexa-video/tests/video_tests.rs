use reciplexa_motion::*;
use reciplexa_raster::RasterOptions;
use reciplexa_scene::{Circle, Color, Document, Page, PaperSize, Shape};
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
