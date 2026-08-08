//! Timeline model for motion preview sync (Phase 8 stub usable by Phase 12).

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct TimeMs(pub u64);

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineTrack {
    pub id: u64,
    pub name: String,
    pub start: TimeMs,
    pub end: TimeMs,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Timeline {
    pub playhead: TimeMs,
    pub duration: TimeMs,
    pub tracks: Vec<TimelineTrack>,
    pub playing: bool,
}

impl Timeline {
    pub fn seek(&mut self, t: TimeMs) {
        self.playhead = TimeMs(t.0.min(self.duration.0));
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn tick(&mut self, delta_ms: u64) {
        if !self.playing {
            return;
        }
        let next = self.playhead.0.saturating_add(delta_ms);
        if next >= self.duration.0 {
            self.playhead = self.duration;
            self.playing = false;
        } else {
            self.playhead = TimeMs(next);
        }
    }

    pub fn active_tracks(&self) -> impl Iterator<Item = &TimelineTrack> {
        let t = self.playhead.0;
        self.tracks
            .iter()
            .filter(move |tr| tr.start.0 <= t && t < tr.end.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
