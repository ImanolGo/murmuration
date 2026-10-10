//! Drawing a frame from the world and camera.
//!
//! For now (M0) there is no world yet: this draws the empty dusk sky over the
//! reed bed.

pub mod sky;

use ratatui::Frame;

/// Draws a single frame to the terminal.
pub fn draw(frame: &mut Frame) {
    sky::draw(frame);
}
