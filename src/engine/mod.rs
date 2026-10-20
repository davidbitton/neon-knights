//! Platform-independent rendering engine: an indexed-color framebuffer with
//! AGI-style visual / priority / control planes, vector drawing primitives,
//! code-defined sprites and a tiny actor movement model.
//!
//! Nothing in here touches the browser, so all of it unit-tests natively.

pub mod actor;
pub mod font;
pub mod gfx;
pub mod palette;
pub mod path;
pub mod sprite;
