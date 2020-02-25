#![allow(dead_code)]

#[macro_use]
extern crate log;

extern crate serde;
extern crate serde_bytes;

extern crate hashcons;

#[macro_use]
pub mod macros;

pub mod adapton;

pub type Glyph = bitmap::Bitmap;
pub type GlyphMap = std::collections::HashMap<types::lang::Name, Glyph>;

pub mod chain;
pub mod grid;
pub mod bitmap;
pub mod menu;
pub mod eval;
pub mod glyph;
pub mod types;
pub mod render;
