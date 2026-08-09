//! Rasterize sampled frames and write PNG sequences.

use std::fs;
use std::path::{Path, PathBuf};

use reciplexa_raster::{frame_to_png, rasterize_page, RasterError, RasterFrame, RasterOptions};
use reciplexa_scene::Document;

use crate::sample::{SampledSequence, VideoLoss, VideoLossKind};
use crate::{VideoError, VideoProfile};

#[derive(Debug, Clone, PartialEq)]
pub struct VideoRasterFrame {
    pub time_ms: u64,
    pub frame: RasterFrame,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoRasterSequence {
    pub profile: VideoProfile,
    pub frames: Vec<VideoRasterFrame>,
    pub losses: Vec<VideoLoss>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteError {
    Io(String),
    Encode(String),
}

/// Rasterize a static document once per sampled frame time (temporal length preview).
/// Losses from rasterization are merged and tagged with the video profile.
pub fn rasterize_sampled_sequence(
    doc: &Document,
    sampled: &SampledSequence,
    page_index: usize,
    opts: &RasterOptions,
) -> Result<VideoRasterSequence, RasterError> {
    let mut frames = Vec::with_capacity(sampled.frames.len());
    let mut losses = sampled.losses.clone();
    for fs in &sampled.frames {
        let frame = rasterize_page(doc, page_index, opts)?;
        for rl in &frame.losses {
            losses.push(VideoLoss {
                kind: VideoLossKind::StrictApproximationSkipped,
                profile: sampled.profile,
                track_id: None,
                detail: format!("raster loss at t={}: {rl:?}", fs.time.0),
            });
        }
        frames.push(VideoRasterFrame {
            time_ms: fs.time.0,
            frame,
        });
    }
    Ok(VideoRasterSequence {
        profile: sampled.profile,
        frames,
        losses,
    })
}

/// Write `frame_00000.png`, `frame_00001.png`, … under `dir`.
pub fn write_png_sequence(
    seq: &VideoRasterSequence,
    dir: &Path,
) -> Result<Vec<PathBuf>, WriteError> {
    fs::create_dir_all(dir).map_err(|e| WriteError::Io(e.to_string()))?;
    let mut paths = Vec::with_capacity(seq.frames.len());
    for (i, vf) in seq.frames.iter().enumerate() {
        let path = dir.join(format!("frame_{i:05}.png"));
        let png = frame_to_png(&vf.frame).map_err(|e| WriteError::Encode(format!("{e:?}")))?;
        fs::write(&path, png).map_err(|e| WriteError::Io(e.to_string()))?;
        paths.push(path);
    }
    Ok(paths)
}

/// Convenience: sample → rasterize → write (Preview/Final options must be supplied).
pub fn export_png_sequence(
    timeline: &reciplexa_motion::MotionTimeline,
    doc: &Document,
    page_index: usize,
    video_opts: &crate::VideoOptions,
    raster_opts: &RasterOptions,
    dir: &Path,
) -> Result<(VideoRasterSequence, Vec<PathBuf>), VideoPipelineError> {
    let sampled =
        crate::sample_timeline_sequence(timeline, video_opts).map_err(VideoPipelineError::Sample)?;
    let seq = rasterize_sampled_sequence(doc, &sampled, page_index, raster_opts)
        .map_err(VideoPipelineError::Raster)?;
    let paths = write_png_sequence(&seq, dir).map_err(VideoPipelineError::Write)?;
    Ok((seq, paths))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoPipelineError {
    Sample(VideoError),
    Raster(RasterError),
    Write(WriteError),
}
