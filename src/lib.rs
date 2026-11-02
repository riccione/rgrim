//! A lightweight screenshot sniper and annotation tool.
//!
//! The library is organised around the windows a capture flows through:
//! [`capture`] grabs a frame of the primary monitor, [`ui`] shows the
//! full-screen region selector, and [`editor`] hosts the annotation
//! canvas with its export actions. [`dashboard`] is the optional GUI
//! entry point, while [`export`] and [`style`] provide shared file
//! naming and egui styling helpers.
//!
//! The `rgrim` binary wires these together; [`VERSION`] exposes the
//! crate version used by the CLI and the windows.

/// The application version, as declared in `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod capture;
pub mod dashboard;
pub mod editor;
pub mod export;
pub mod style;
pub mod ui;
