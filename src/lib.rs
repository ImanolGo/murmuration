//! murmuration — starlings at dusk, in your terminal.
//!
//! The simulation will live in `sim` and will know nothing about the terminal:
//! it is pure, deterministic for a given seed, and runs headless. Only
//! [`render`] and the binary touch the terminal.

pub mod render;
