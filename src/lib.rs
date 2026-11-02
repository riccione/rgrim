/// The application version, as declared in `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod capture;
pub mod dashboard;
pub mod editor;
pub mod export;
pub mod style;
pub mod ui;
