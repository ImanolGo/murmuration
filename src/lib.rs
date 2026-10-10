//! murmuration — starlings at dusk, in your terminal.
//!
//! murmuration is a live, three-dimensional flock of starlings watched from the
//! ground and drawn in braille silhouettes against an evening sky. The
//! simulation lives in [`sim`] and knows nothing about the terminal: it is pure,
//! deterministic for a given seed, and runs headless. Only [`render`] and the
//! binary touch the terminal.
//!
//! # Stability
//!
//! The library exists for docs and tests; it has no stability guarantee. The
//! product is the `murmuration` binary, and only its command-line interface and
//! flags are meant to be stable. These modules are an implementation detail and
//! may change in any release.

#[doc(hidden)]
pub mod cli;
#[doc(hidden)]
pub mod render;
#[doc(hidden)]
pub mod sim;
