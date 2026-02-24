pub mod analogue;
pub mod digital;
pub mod subclock;

use crate::canvas::{Canvas, FontState};
use crate::config::ClockConfig;
use crate::time_utils::ClockTime;

#[allow(dead_code)]
pub struct ClockState {
    pub config: ClockConfig,
    pub time: ClockTime,
    pub compact: bool,
    pub width: u32,
    pub height: u32,
}

pub fn render(canvas: &mut Canvas, state: &ClockState, font: &FontState) {
    use crate::config::FaceMode;

    match state.config.clock.face {
        FaceMode::Digital => digital::render(canvas, state, font),
        FaceMode::Analogue => analogue::render(canvas, state, font),
    }

    // Draw subclocks
    if !state.config.timezone.is_empty() {
        subclock::render(canvas, state, font);
    }
}
