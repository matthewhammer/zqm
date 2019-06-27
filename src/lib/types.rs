
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
pub enum CliCommand {
    Version,
    Completions(String),
    // to do: make a simple module for text entry/editing,
    // and/or, use existing readline library
    ReadLine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    CliCommand(CliCommand),
    MakeTime(Name),
    MakePlace(Name),
    GotoPlace(Name),
    
    Bitmap(super::bitmap::Command),
    Save,
    Restore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Dir2D {
    Up,
    Down,
    Left,
    Right
}

/// store and explore a history of all issued commands
pub mod command_history {
    use std::rc::Rc;
    use super::{Command, Nat};

    pub type Commands = Vec<Command>;

    pub enum History {
        Empty,
        Linear(Commands),
        Fork(Fork),
    }

    pub struct Fork {
        pub shared_history: Rc<Commands>,
        pub sub_histories: Vec<Rc<History>>,
    }

    pub enum Context {
        Empty,
        Linear(Rc<Context>, Commands),
        Fork(Rc<Context>, Fork, Nat),
    }

    pub struct Cursor {
        pub context: Context,
        pub history: History,
    }
}
