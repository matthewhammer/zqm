// Serde: Persistent state between invocations of ZQM
use serde::{Deserialize, Serialize};
use types::{Media, Name, Dir1D};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub name: Option<Name>,
    pub media: Box<Media>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Chain {
    Empty,
    Node(Node, Box<Chain>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AutoCommand {
    InsertStart(Media),
    DeleteStart,
    InsertEnd(Media),
    DeleteEnd,
    Replace(Name, Media)
    InsertAfter(Name, Media),
    DeleteAfter(Name),
    InsertBefore(Name, Media),
    DeleteBefore(Name),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Editor {
    pub head: Chain,
    pub cursor: Option<Node>,
    pub tail: Chain,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InitCommand {
    Empty,
    String(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EditCommand {
    MoveRel(Dir1D),
    MoveAbs(usize),
    MoveBegin,
    MoveEnd,
}

/// commands that advance the evolution of a bitmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// commands that create new bitmaps
    Init(InitCommand),

    /// commands that advance the state of the bitmap,
    /// whose execution is independent of editor state
    Auto(AutoCommand),

    /// commands that advance the editor state,
    /// and possibly, its associated bitmap state
    Edit(EditCommand),
}
