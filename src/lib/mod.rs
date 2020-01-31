#[macro_use] extern crate log;

extern crate serde;
extern crate serde_bytes;

// SDL: Keyboard/mouse input events, multi-media output abstractions:
extern crate sdl2;

extern crate hashcons;

#[macro_use]
pub mod macros;

pub type Glyph = bitmap::Bitmap;
pub type GlyphMap = std::collections::HashMap<types::Name, Glyph>;

pub mod chain;
pub mod grid;

pub mod types;
pub mod eval;
pub mod bitmap;
pub mod glyph;
