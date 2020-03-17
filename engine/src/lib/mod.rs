#![allow(dead_code)]

#[macro_use]
extern crate log;

extern crate hashcons;
extern crate serde;
extern crate serde_bytes;

extern crate ic_http_agent;
extern crate serde_idl;

extern crate tokio;

pub mod eval;
pub mod init;
pub mod types;

// to do: complete menu module
pub mod menu;

pub mod bitmap;

// to do: complete adapton module:
pub mod adapton;

// to do: complete chain and grid modules:
pub mod chain;
pub mod grid;

// -------- graphical output ---------

#[macro_use]
pub mod macros;
pub type Glyph = bitmap::Bitmap;
pub type GlyphMap = std::collections::HashMap<types::lang::Name, Glyph>;

pub mod glyph;
pub mod render;

// -------- Candid (IC input/output) ---------

pub mod candid;
