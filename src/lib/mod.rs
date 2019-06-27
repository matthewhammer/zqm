#[macro_use] extern crate log;

extern crate serde;
extern crate serde_bytes;

// SDL: Keyboard/mouse input events, multi-media output abstractions:
extern crate sdl2;

// Serde: Persistent state between invocations of ZQM
use serde::{Deserialize, Serialize};

pub type Nat = usize;

pub type Name = String; // to do

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameFn {  // to do
    pub path: Vec<Name>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Point {
    pub time:  Name,
    pub place: Name,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Space {
    pub time:  NameFn,
    pub place: NameFn,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Locus {
    pub point: Point,
    pub space: Space,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Version,
    Completions(String),
    MakeTime(Name),
    MakePlace(Name),
    GotoPlace(Name),
    ReadLine,
    Bitmap(bitmap::Command),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Dir2D {
    Up,
    Down,
    Left,
    Right
}

pub mod eval;
pub mod bitmap;
