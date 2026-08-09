use reciplexa_motion::*;

#[test]
fn constant_and_keyframe_sample_determinism() {
    let c = MotionTrack::constant(3.5);
    assert_eq!(sample_f64(&c, TimeMs(0), SampleMode::Strict).unwrap(), 3.5);
    assert_eq!(sample_f64(&c, TimeMs(999), SampleMode::Preview).unwrap(), 3.5);

    let kf = MotionTrack::keyframes(vec![
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
    ]);
    assert_eq!(sample_f64(&kf, TimeMs(0), SampleMode::Strict).unwrap(), 0.0);
    assert_eq!(sample_f64(&kf, TimeMs(1000), SampleMode::Strict).unwrap(), 10.0);
    assert!(matches!(
        sample_f64(&kf, TimeMs(500), SampleMode::Strict),
        Err(SampleError::StrictApproximation)
    ));
    let mid = sample_f64(&kf, TimeMs(500), SampleMode::Preview).unwrap();
    assert!((mid - 5.0).abs() < 1e-9);
}

#[test]
fn samples_rate_and_validation() {
    let track = MotionTrack::Samples {
        rate_hz: 10.0,
        values: vec![1.0, 2.0, 3.0],
    };
    validate_track_f64(&track).unwrap();
    assert_eq!(
        sample_f64(&track, TimeMs(0), SampleMode::Strict).unwrap(),
        1.0
    );
    assert_eq!(
        sample_f64(&track, TimeMs(100), SampleMode::Strict).unwrap(),
        2.0
    );

    assert!(validate_track_f64(&MotionTrack::constant(f64::NAN)).is_err());
    assert!(validate_track_f64(&MotionTrack::Keyframes(vec![])).is_err());
    assert!(validate_source_range(TimeMs(5), TimeMs(5), RangePolicy::Loop).is_err());
}

#[test]
fn time_transform_and_range_policy() {
    assert_eq!(
        TimeTransform::Offset(DurationMs(100)).apply(TimeMs(50)),
        TimeMs(150)
    );
    assert_eq!(
        TimeTransform::Scale { factor: 2.0 }.apply(TimeMs(100)),
        TimeMs(200)
    );
    assert_eq!(
        TimeTransform::Reverse {
            source_len: DurationMs(1000)
        }
        .apply(TimeMs(250)),
        TimeMs(750)
    );
    let mapped =
        apply_range_policy(TimeMs(1500), TimeMs(0), TimeMs(1000), RangePolicy::Loop).unwrap();
    assert_eq!(mapped, Some(TimeMs(500)));
    assert!(apply_range_policy(TimeMs(150), TimeMs(0), TimeMs(100), RangePolicy::Failure).is_err());
}

#[test]
fn timeline_tick_and_seek() {
    let mut tl = MotionTimeline {
        playhead: TimeMs::ZERO,
        duration: DurationMs(1000),
        tracks: vec![],
        playing: true,
    };
    tl.tick(400);
    assert_eq!(tl.playhead, TimeMs(400));
    tl.tick(700);
    assert_eq!(tl.playhead, TimeMs(1000));
    assert!(!tl.playing);
    tl.seek(TimeMs(200));
    assert_eq!(tl.playhead, TimeMs(200));
}

#[test]
fn easing_partitions() {
    assert_eq!(Easing::Linear.apply(0.5), 0.5);
    assert!(Easing::EaseInQuad.apply(0.5) < 0.5);
    assert!(Easing::EaseOutQuad.apply(0.5) > 0.5);
    let mid = Easing::EaseInOutCubic.apply(0.5);
    assert!((mid - 0.5).abs() < 1e-9);
}
