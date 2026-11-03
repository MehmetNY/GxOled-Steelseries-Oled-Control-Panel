//! GxOled GIF Studio engine. Platform independent: no windows, no device access.
//!
//! A GIF is decoded to RGBA frames ([`Source`]), each frame is placed on the OLED and converted to 1 bit with
//! the user's [`Settings`] ([`process`]), and the result can be played on the OLED, exported as a two-colour
//! GIF, or saved as a [`Project`].
//!
//! The settings model and the tone / dither maths follow GGIF-Studio (MIT, (c) 2026 Abdelnour Osman,
//! <https://github.com/D4rkTT/GGIF-Studio>); this crate is an independent Rust reimplementation.

pub mod clock;
pub mod export;
pub mod plan;
pub mod process;
pub mod project;
pub mod settings;
pub mod source;
pub mod studio;

pub use clock::PreviewClock;
pub use project::Project;
pub use settings::{Background, Dither, Fit, Param, Settings};
pub use source::Source;
pub use studio::Studio;
