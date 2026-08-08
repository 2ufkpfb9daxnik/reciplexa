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
}
