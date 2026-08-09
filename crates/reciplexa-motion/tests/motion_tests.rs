use reciplexa_motion::sample::ease;
use reciplexa_motion::*;

#[test]
fn constant_and_keyframe_sample_determinism() {
    let c = MotionTrack::constant(3.5);
    assert_eq!(sample_f64(&c, TimeMs(0), SampleMode::Strict).unwrap(), 3.5);
    assert_eq!(
        sample_f64(&c, TimeMs(999), SampleMode::Preview).unwrap(),
        3.5
    );

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
    assert_eq!(
        sample_f64(&kf, TimeMs(1000), SampleMode::Strict).unwrap(),
        10.0
    );
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
    // EaseInOutCubic first half (t < 0.5).
    let early = Easing::EaseInOutCubic.apply(0.25);
    assert!(early < 0.5);
    // Clamp outside [0, 1].
    assert_eq!(Easing::Linear.apply(-1.0), 0.0);
    assert_eq!(Easing::Linear.apply(2.0), 1.0);
    assert_eq!(ease(Easing::EaseInQuad, 0.5), Easing::EaseInQuad.apply(0.5));
}

#[test]
fn time_ms_and_duration_helpers() {
    assert_eq!(TimeMs(100).saturating_add(DurationMs(50)), TimeMs(150));
    assert_eq!(TimeMs(100).saturating_sub(DurationMs(40)), TimeMs(60));
    assert_eq!(TimeMs(10).saturating_sub(DurationMs(40)), TimeMs(0));
    assert!((TimeMs(1500).as_secs_f64() - 1.5).abs() < 1e-12);
    assert_eq!(DurationMs::from_secs_f64(1.5), Some(DurationMs(1500)));
    assert_eq!(DurationMs::from_secs_f64(0.0), Some(DurationMs(0)));
    assert_eq!(DurationMs::from_secs_f64(-1.0), None);
    assert_eq!(DurationMs::from_secs_f64(f64::NAN), None);
    assert_eq!(DurationMs::from_secs_f64(f64::INFINITY), None);
    assert_eq!(DurationMs::ZERO, DurationMs(0));
}

#[test]
fn sample_error_and_edge_paths() {
    assert!(matches!(
        sample_f64(
            &MotionTrack::constant(f64::NAN),
            TimeMs(0),
            SampleMode::Strict
        ),
        Err(SampleError::NonFiniteValue)
    ));
    assert!(matches!(
        sample_f64(
            &MotionTrack::Keyframes(vec![]),
            TimeMs(0),
            SampleMode::Preview
        ),
        Err(SampleError::EmptyKeyframes)
    ));
    assert!(matches!(
        sample_f64(
            &MotionTrack::keyframes(vec![Keyframe {
                at: TimeMs(0),
                value: f64::INFINITY,
                easing_to_next: Easing::Linear,
            }]),
            TimeMs(0),
            SampleMode::Strict
        ),
        Err(SampleError::NonFiniteValue)
    ));

    // Three keyframes: middle exact hit via a.at / b.at arms, skip non-matching window.
    let kf3 = MotionTrack::keyframes(vec![
        Keyframe {
            at: TimeMs(0),
            value: 0.0,
            easing_to_next: Easing::EaseInQuad,
        },
        Keyframe {
            at: TimeMs(500),
            value: 5.0,
            easing_to_next: Easing::EaseOutQuad,
        },
        Keyframe {
            at: TimeMs(1000),
            value: 10.0,
            easing_to_next: Easing::Linear,
        },
    ]);
    assert_eq!(
        sample_f64(&kf3, TimeMs(500), SampleMode::Strict).unwrap(),
        5.0
    );
    // Beyond last holds last value.
    assert_eq!(
        sample_f64(&kf3, TimeMs(2000), SampleMode::Strict).unwrap(),
        10.0
    );
    // Before first holds first.
    assert_eq!(
        sample_f64(&kf3, TimeMs(0), SampleMode::Preview).unwrap(),
        0.0
    );
    let eased = sample_f64(&kf3, TimeMs(250), SampleMode::Preview).unwrap();
    assert!(eased > 0.0 && eased < 5.0);

    // Single keyframe: no windows → post-loop hold.
    let one_kf = MotionTrack::keyframes(vec![Keyframe {
        at: TimeMs(10),
        value: 4.0,
        easing_to_next: Easing::Linear,
    }]);
    assert_eq!(
        sample_f64(&one_kf, TimeMs(10), SampleMode::Strict).unwrap(),
        4.0
    );
    assert_eq!(
        sample_f64(&one_kf, TimeMs(99), SampleMode::Preview).unwrap(),
        4.0
    );

    assert!(matches!(
        sample_f64(
            &MotionTrack::Samples {
                rate_hz: f64::NAN,
                values: vec![1.0],
            },
            TimeMs(0),
            SampleMode::Strict
        ),
        Err(SampleError::NonFiniteRate)
    ));
    assert!(matches!(
        sample_f64(
            &MotionTrack::Samples {
                rate_hz: 0.0,
                values: vec![1.0],
            },
            TimeMs(0),
            SampleMode::Strict
        ),
        Err(SampleError::NonFiniteRate)
    ));
    assert!(matches!(
        sample_f64(
            &MotionTrack::Samples {
                rate_hz: 10.0,
                values: vec![],
            },
            TimeMs(0),
            SampleMode::Strict
        ),
        Err(SampleError::EmptySamples)
    ));
    assert!(matches!(
        sample_f64(
            &MotionTrack::Samples {
                rate_hz: 10.0,
                values: vec![1.0, f64::NAN],
            },
            TimeMs(0),
            SampleMode::Strict
        ),
        Err(SampleError::NonFiniteValue)
    ));

    let samples = MotionTrack::Samples {
        rate_hz: 10.0,
        values: vec![1.0, 2.0, 3.0],
    };
    // Past end holds last.
    assert_eq!(
        sample_f64(&samples, TimeMs(5000), SampleMode::Strict).unwrap(),
        3.0
    );
    // Preview lerp between bins.
    let lerp = sample_f64(&samples, TimeMs(50), SampleMode::Preview).unwrap();
    assert!((lerp - 1.5).abs() < 1e-9);
    // Preview at last bin (no next).
    assert_eq!(
        sample_f64(&samples, TimeMs(200), SampleMode::Preview).unwrap(),
        3.0
    );
    // Single-sample preview last-bin path.
    let one = MotionTrack::Samples {
        rate_hz: 10.0,
        values: vec![7.0],
    };
    assert_eq!(
        sample_f64(&one, TimeMs(0), SampleMode::Preview).unwrap(),
        7.0
    );
}

#[test]
fn timeline_play_pause_span_and_idle_tick() {
    let placement = TemporalPlacement::span(TimeMs(100), TimeMs(600));
    assert_eq!(placement.parent_start, TimeMs(100));
    assert_eq!(placement.parent_end, TimeMs(600));
    assert_eq!(placement.child_source_start, TimeMs::ZERO);
    assert_eq!(placement.child_source_end, TimeMs(500));
    assert_eq!(placement.transform, TimeTransform::Identity);
    assert_eq!(placement.range_policy, RangePolicy::Hold);

    let mut tl = MotionTimeline {
        playhead: TimeMs(10),
        duration: DurationMs(1000),
        tracks: vec![],
        playing: false,
    };
    tl.tick(50);
    assert_eq!(tl.playhead, TimeMs(10));
    tl.play();
    assert!(tl.playing);
    tl.tick(20);
    assert_eq!(tl.playhead, TimeMs(30));
    tl.pause();
    assert!(!tl.playing);
    tl.seek(TimeMs(9999));
    assert_eq!(tl.playhead, TimeMs(1000));
}

#[test]
fn time_transform_all_arms() {
    assert_eq!(TimeTransform::Identity.apply(TimeMs(42)), TimeMs(42));
    assert_eq!(
        TimeTransform::Freeze(TimeMs(7)).apply(TimeMs(100)),
        TimeMs(7)
    );
    assert_eq!(
        TimeTransform::Reverse {
            source_len: DurationMs(100)
        }
        .apply(TimeMs(100)),
        TimeMs(0)
    );
    assert_eq!(
        TimeTransform::Reverse {
            source_len: DurationMs(100)
        }
        .apply(TimeMs(150)),
        TimeMs(0)
    );
    // Scale with non-positive rounded result.
    assert_eq!(
        TimeTransform::Scale { factor: 0.0001 }.apply(TimeMs(1)),
        TimeMs(0)
    );
    let composed = TimeTransform::Compose(
        Box::new(TimeTransform::Offset(DurationMs(10))),
        Box::new(TimeTransform::Scale { factor: 2.0 }),
    );
    assert_eq!(composed.apply(TimeMs(5)), TimeMs(20)); // (5*2)+10
}

#[test]
fn range_policy_all_arms() {
    assert!(matches!(
        apply_range_policy(TimeMs(0), TimeMs(10), TimeMs(10), RangePolicy::Hold),
        Err(RangePolicyError::EmptySourceRange)
    ));

    // In-range → local offset.
    assert_eq!(
        apply_range_policy(TimeMs(30), TimeMs(10), TimeMs(50), RangePolicy::Transparent).unwrap(),
        Some(TimeMs(20))
    );

    assert_eq!(
        apply_range_policy(TimeMs(5), TimeMs(10), TimeMs(50), RangePolicy::Transparent).unwrap(),
        None
    );
    assert_eq!(
        apply_range_policy(TimeMs(60), TimeMs(10), TimeMs(50), RangePolicy::Transparent).unwrap(),
        None
    );

    assert_eq!(
        apply_range_policy(TimeMs(5), TimeMs(10), TimeMs(50), RangePolicy::Hold).unwrap(),
        Some(TimeMs(0))
    );
    assert_eq!(
        apply_range_policy(TimeMs(60), TimeMs(10), TimeMs(50), RangePolicy::Hold).unwrap(),
        Some(TimeMs(39))
    );

    // Loop before start: back != 0.
    assert_eq!(
        apply_range_policy(TimeMs(5), TimeMs(10), TimeMs(50), RangePolicy::Loop).unwrap(),
        Some(TimeMs(35))
    );
    // start >= len so t = start - len is still before start and back == 0.
    assert_eq!(
        apply_range_policy(TimeMs(10), TimeMs(50), TimeMs(90), RangePolicy::Loop).unwrap(),
        Some(TimeMs(0))
    );

    // PingPong forward and reverse halves; before-start raw=0.
    assert_eq!(
        apply_range_policy(TimeMs(5), TimeMs(10), TimeMs(50), RangePolicy::PingPong).unwrap(),
        Some(TimeMs(0))
    );
    // len=40, cycle=78; at start+50 → raw=50 >= len → reverse.
    assert_eq!(
        apply_range_policy(TimeMs(60), TimeMs(10), TimeMs(50), RangePolicy::PingPong).unwrap(),
        Some(TimeMs(28))
    );
    // Tiny range: cycle.max(1).
    assert_eq!(
        apply_range_policy(TimeMs(5), TimeMs(0), TimeMs(1), RangePolicy::PingPong).unwrap(),
        Some(TimeMs(0))
    );
}

#[test]
fn validate_track_transform_and_range() {
    validate_track_f64(&MotionTrack::constant(1.0)).unwrap();

    let sorted = MotionTrack::keyframes(vec![
        Keyframe {
            at: TimeMs(0),
            value: 1.0,
            easing_to_next: Easing::Linear,
        },
        Keyframe {
            at: TimeMs(10),
            value: 2.0,
            easing_to_next: Easing::Linear,
        },
    ]);
    validate_track_f64(&sorted).unwrap();

    assert!(matches!(
        validate_track_f64(&MotionTrack::keyframes(vec![Keyframe {
            at: TimeMs(0),
            value: f64::NAN,
            easing_to_next: Easing::Linear,
        }])),
        Err(MotionValidationError::NonFiniteNumber("keyframe"))
    ));
    assert!(matches!(
        validate_track_f64(&MotionTrack::keyframes(vec![
            Keyframe {
                at: TimeMs(10),
                value: 1.0,
                easing_to_next: Easing::Linear,
            },
            Keyframe {
                at: TimeMs(5),
                value: 2.0,
                easing_to_next: Easing::Linear,
            },
        ])),
        Err(MotionValidationError::UnsortedKeyframes)
    ));

    assert!(matches!(
        validate_track_f64(&MotionTrack::Samples {
            rate_hz: -1.0,
            values: vec![1.0],
        }),
        Err(MotionValidationError::BadSampleRate)
    ));
    assert!(matches!(
        validate_track_f64(&MotionTrack::Samples {
            rate_hz: f64::NAN,
            values: vec![1.0],
        }),
        Err(MotionValidationError::BadSampleRate)
    ));
    assert!(matches!(
        validate_track_f64(&MotionTrack::Samples {
            rate_hz: 10.0,
            values: vec![],
        }),
        Err(MotionValidationError::EmptySamples)
    ));
    assert!(matches!(
        validate_track_f64(&MotionTrack::Samples {
            rate_hz: 10.0,
            values: vec![1.0, f64::INFINITY],
        }),
        Err(MotionValidationError::NonFiniteNumber("sample"))
    ));

    validate_transform(&TimeTransform::Identity).unwrap();
    validate_transform(&TimeTransform::Offset(DurationMs(1))).unwrap();
    validate_transform(&TimeTransform::Reverse {
        source_len: DurationMs(1),
    })
    .unwrap();
    validate_transform(&TimeTransform::Freeze(TimeMs(0))).unwrap();
    validate_transform(&TimeTransform::Scale { factor: 1.5 }).unwrap();
    assert!(matches!(
        validate_transform(&TimeTransform::Scale { factor: 0.0 }),
        Err(MotionValidationError::ScaleFactorInvalid)
    ));
    assert!(matches!(
        validate_transform(&TimeTransform::Scale { factor: -1.0 }),
        Err(MotionValidationError::ScaleFactorInvalid)
    ));
    assert!(matches!(
        validate_transform(&TimeTransform::Scale { factor: f64::NAN }),
        Err(MotionValidationError::ScaleFactorInvalid)
    ));
    validate_transform(&TimeTransform::Compose(
        Box::new(TimeTransform::Identity),
        Box::new(TimeTransform::Offset(DurationMs(2))),
    ))
    .unwrap();
    assert!(matches!(
        validate_transform(&TimeTransform::Compose(
            Box::new(TimeTransform::Scale { factor: -1.0 }),
            Box::new(TimeTransform::Identity),
        )),
        Err(MotionValidationError::ScaleFactorInvalid)
    ));

    validate_source_range(TimeMs(0), TimeMs(10), RangePolicy::Hold).unwrap();
    validate_source_range(TimeMs(0), TimeMs(10), RangePolicy::Transparent).unwrap();
    assert!(matches!(
        validate_source_range(TimeMs(10), TimeMs(10), RangePolicy::Failure),
        Err(MotionValidationError::EmptySourceRange)
    ));
}

#[test]
fn preview_sync_playhead_interpolates() {
    let mut tl = MotionTimeline {
        playhead: TimeMs(500),
        duration: DurationMs(1000),
        tracks: vec![TimelineTrack {
            id: 1,
            node: None,
            name: "opacity".into(),
            placement: TemporalPlacement::span(TimeMs(0), TimeMs(1000)),
            track: MotionTrack::keyframes(vec![
                Keyframe {
                    at: TimeMs(0),
                    value: 0.0,
                    easing_to_next: Easing::Linear,
                },
                Keyframe {
                    at: TimeMs(1000),
                    value: 1.0,
                    easing_to_next: Easing::Linear,
                },
            ]),
        }],
        playing: false,
    };
    let preview = sync_playhead(&tl, SyncProfile::Preview);
    assert_eq!(preview.profile, SyncProfile::Preview);
    assert!((preview.values[0].value.unwrap() - 0.5).abs() < 1e-9);

    let final_s = sync_at(&tl, TimeMs(500), SyncProfile::Final);
    assert_eq!(
        final_s.values[0].error,
        Some(SampleError::StrictApproximation)
    );

    let diff = compare_preview_final(&tl);
    assert!(diff.divergent_track_ids.contains(&1));

    tl.seek(TimeMs(0));
    let at_start = sync_playhead(&tl, SyncProfile::Final);
    assert_eq!(at_start.values[0].value, Some(0.0));
}

#[test]
fn preview_sync_transparent_and_failure_range() {
    let mut placement = TemporalPlacement::span(TimeMs(100), TimeMs(200));
    placement.range_policy = RangePolicy::Transparent;
    let tl = MotionTimeline {
        playhead: TimeMs(50),
        duration: DurationMs(300),
        tracks: vec![TimelineTrack {
            id: 2,
            node: None,
            name: "t".into(),
            placement,
            track: MotionTrack::constant(3.0),
        }],
        playing: false,
    };
    let sync = sync_playhead(&tl, SyncProfile::Preview);
    assert_eq!(sync.values[0].value, None);
    assert_eq!(sync.values[0].error, None);

    let mut bad = TemporalPlacement::span(TimeMs(0), TimeMs(1));
    bad.parent_start = TimeMs(10);
    bad.parent_end = TimeMs(10);
    bad.range_policy = RangePolicy::Failure;
    let tl2 = MotionTimeline {
        playhead: TimeMs(5),
        duration: DurationMs(20),
        tracks: vec![TimelineTrack {
            id: 4,
            node: None,
            name: "bad".into(),
            placement: bad,
            track: MotionTrack::constant(1.0),
        }],
        playing: false,
    };
    let sync2 = sync_playhead(&tl2, SyncProfile::Preview);
    assert_eq!(sync2.values[0].error, Some(SampleError::EmptyKeyframes));
}

#[test]
fn compare_preview_final_aligned_constants() {
    let tl = MotionTimeline {
        playhead: TimeMs(10),
        duration: DurationMs(100),
        tracks: vec![TimelineTrack {
            id: 1,
            node: None,
            name: "c".into(),
            placement: TemporalPlacement::span(TimeMs(0), TimeMs(100)),
            track: MotionTrack::constant(2.5),
        }],
        playing: false,
    };
    let diff = compare_preview_final(&tl);
    assert!(diff.divergent_track_ids.is_empty());
    assert_eq!(diff.preview.values[0].value, Some(2.5));
    assert_eq!(diff.final_sync.values[0].value, Some(2.5));
}

#[test]
fn compare_preview_final_sample_rate_diverges() {
    let tl = MotionTimeline {
        playhead: TimeMs(500),
        duration: DurationMs(2000),
        tracks: vec![TimelineTrack {
            id: 5,
            node: None,
            name: "s".into(),
            placement: TemporalPlacement::span(TimeMs(0), TimeMs(2000)),
            track: MotionTrack::Samples {
                rate_hz: 1.0,
                values: vec![0.0, 10.0],
            },
        }],
        playing: false,
    };
    let diff = compare_preview_final(&tl);
    assert!(diff.divergent_track_ids.contains(&5));
    assert!((diff.preview.values[0].value.unwrap() - 5.0).abs() < 1e-9);
    assert_eq!(diff.final_sync.values[0].value, Some(0.0));
}

#[test]
fn compare_preview_final_both_none_transparent() {
    let mut placement = TemporalPlacement::span(TimeMs(100), TimeMs(200));
    placement.range_policy = RangePolicy::Transparent;
    let tl = MotionTimeline {
        playhead: TimeMs(50),
        duration: DurationMs(300),
        tracks: vec![TimelineTrack {
            id: 9,
            node: None,
            name: "out".into(),
            placement,
            track: MotionTrack::constant(1.0),
        }],
        playing: false,
    };
    let diff = compare_preview_final(&tl);
    assert!(diff.preview.values[0].value.is_none());
    assert!(diff.final_sync.values[0].value.is_none());
    assert!(diff.divergent_track_ids.is_empty());
}
