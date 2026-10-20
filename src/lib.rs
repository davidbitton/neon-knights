//! Neon Nights: A Gentleman's Evening Out.
//!
//! An original parody adventure in the style of late-1980s parser-driven
//! adventure games. The crate is split into:
//! * [`engine`] – platform-independent framebuffer, vector drawing, priority
//!   and walkability planes, sprites, actor movement;
//! * [`game`]   – pure game logic: parser, state, scoring, inventory, scene
//!   content and all the writing;
//! * `web`      – the thin wasm/web-sys layer (only compiled for wasm32).

pub mod engine;
pub mod game;

#[cfg(target_arch = "wasm32")]
mod web;
