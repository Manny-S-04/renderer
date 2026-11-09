#![allow(clippy::needless_range_loop)]

pub mod canvas;
pub mod color;
pub mod demos;
pub mod event;
pub mod math;

#[cfg(all(feature = "window", unix))]
#[path = "window_x11.rs"]
pub mod window;

#[cfg(all(feature = "window", windows))]
#[path = "window_win32.rs"]
pub mod window;
