//! The app bundles hosts embed. `generated.rs` is written by
//! `cargo run -p fri3d-pack` and lists `LAUNCHER` plus `APPS` in launcher
//! order. Run the pack tool before building a host.
#![no_std]

include!("generated.rs");
