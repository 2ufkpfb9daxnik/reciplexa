//! Reference Client standard packages (`rpx.std.*`) — Phase 14.
//!
//! These types are a Strangler-friendly façade over scene / identity / motion
//! so Source, GUI, and Backend can share a small typed API before full Package
//! Manifest wiring lands.

#![forbid(unsafe_code)]

pub mod core;

pub use core::{Angle, Color, Length, Point, Range, Rect, Size};
