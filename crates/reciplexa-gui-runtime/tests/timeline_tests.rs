use reciplexa_gui_runtime::timeline::*;

#[test]
fn tick_advances_while_playing() {
    let mut tl = Timeline {
        playhead: TimeMs(0),
        duration: TimeMs(1000),
        tracks: vec![TimelineTrack {
            id: 1,
            name: "fade".into(),
            start: TimeMs(0),
            end: TimeMs(500),
        }],
        playing: true,
    };
    tl.tick(100);
    assert_eq!(tl.playhead, TimeMs(100));
    assert_eq!(tl.active_tracks().count(), 1);
}

#[test]
fn seek_clamps_to_duration() {
    let mut tl = Timeline {
        playhead: TimeMs(0),
        duration: TimeMs(500),
        tracks: vec![],
        playing: false,
    };
    tl.seek(TimeMs(999));
    assert_eq!(tl.playhead, TimeMs(500));
    tl.seek(TimeMs(100));
    assert_eq!(tl.playhead, TimeMs(100));
}

#[test]
fn pause_prevents_tick() {
    let mut tl = Timeline {
        playhead: TimeMs(50),
        duration: TimeMs(1000),
        tracks: vec![],
        playing: false,
    };
    tl.tick(100);
    assert_eq!(tl.playhead, TimeMs(50));
}

#[test]
fn tick_to_end_stops_playing() {
    let mut tl = Timeline {
        playhead: TimeMs(900),
        duration: TimeMs(1000),
        tracks: vec![],
        playing: true,
    };
    tl.tick(200);
    assert_eq!(tl.playhead, TimeMs(1000));
    assert!(!tl.playing);
}

#[test]
fn active_tracks_half_open_interval() {
    let tl = Timeline {
        playhead: TimeMs(500),
        duration: TimeMs(1000),
        tracks: vec![
            TimelineTrack {
                id: 1,
                name: "before".into(),
                start: TimeMs(0),
                end: TimeMs(500),
            },
            TimelineTrack {
                id: 2,
                name: "at_end".into(),
                start: TimeMs(0),
                end: TimeMs(500),
            },
            TimelineTrack {
                id: 3,
                name: "after".into(),
                start: TimeMs(500),
                end: TimeMs(1000),
            },
        ],
        playing: false,
    };
    let active: Vec<_> = tl.active_tracks().map(|t| t.id).collect();
    assert!(!active.contains(&1));
    assert!(!active.contains(&2));
    assert!(!tl.playing);
}

#[test]
fn play_and_tick_mid_timeline() {
    let mut tl = Timeline {
        playhead: TimeMs(100),
        duration: TimeMs(1000),
        tracks: vec![],
        playing: false,
    };
    tl.play();
    assert!(tl.playing);
    tl.tick(50);
    assert_eq!(tl.playhead, TimeMs(150));
    assert!(tl.playing);
}

#[test]
fn pause_stops_playback() {
    let mut tl = Timeline {
        playhead: TimeMs(10),
        duration: TimeMs(1000),
        tracks: vec![],
        playing: true,
    };
    tl.pause();
    assert!(!tl.playing);
    tl.tick(50);
    assert_eq!(tl.playhead, TimeMs(10));
}
