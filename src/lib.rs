//! murmuration — starlings at dusk, in your terminal.
//!
//! The simulation lives in [`sim`] and knows nothing about the terminal: it is
//! pure, deterministic for a given seed, and runs headless. Only [`render`] and
//! the binary touch the terminal.

pub mod render;
pub mod sim;
