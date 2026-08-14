//! Library surface for goldfinch.
//!
//! The binary in `main.rs` is a thin CLI wrapper over these modules; exposing
//! them as a library lets integration tests exercise the real rendering and
//! fetching code paths directly.

pub mod aws;
pub mod cli;
pub mod commands;
