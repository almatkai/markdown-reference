//! Byte-exact beta common/SDK/session fixtures, with in-memory installation
//! routing and a Tauri boundary that cannot touch the OS.
#![allow(dead_code)] // Unused APIs belong to the unmodified host fixtures.
pub mod native_core;
pub use native_core::extensions::Sessions;
