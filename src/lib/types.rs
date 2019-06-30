
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

    // save/restore manages editor state via an ambient stack of past states
    Save,
    Restore,

    // undo/redo manages editor states via an ambient n-ary tree zipper of past commands,
    // and associated states (think "emacs undo tree").
    Undo,
    Redo,

    // how do save/restore and undo/redo interact?
    // do undo or redo alter the stack?
    // do save or restore alter the tree, or the position in it?

    // the answer is all/none of the above!
    // we should experiment and see.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Dir2D {
    Up,
    Down,
    Left,
    Right
}


/// generalizes the relationship between:
///
/// - Commands
///
/// - Each "non-historical" Command's semantics (e.g., in bitmap
/// editor: move the cursor, toggle bits, etc),
///
/// - their "History" (the ambient stack or tree of past editor
/// states),
///
/// - Each "historical" Command's semantics, e.g., push/pop editor
/// states; undo/redo editor states, including the _mathematical_
/// meaning of "undo/redo", which generalizes and nests.
///
use std::hash::Hash;
use std::fmt::Debug;

trait CommandHistory : Eq + Hash + Debug {
    /// State of an abstracted "editor"
    type State    : Eq + Hash + Debug + Clone;
    /// Commands of an abstracted "editor"
    type Command  : Eq + Hash + Debug;
    /// Historical (re-)focusing commands of an abstracted "editor"
    type HCommand : Eq + Hash + Debug;
    /// Ok responses for HCommand evaluation
    type Resp     : Eq + Hash + Debug;
    /// Err responses for HCommand evaluation
    type Error    : Eq + Hash + Debug;

    /// project/translate any historical command from the (more general) command type
    ///
    /// the environment uses this operation to test whether the
    /// command is recognized/defined, and should be evaluated here.
    ///
    /// to do: enforce "well-formed ensembles", where no two distinct
    /// command histories translate the same command; "well-formed
    /// ensembles" enjoy nice properties: composition order does not
    /// matter, and there are no (potentially very strange) "double
    /// effects" of any one command.
    fn translate_command(&mut self, &Self::Command) -> Option<Self::HCommand>;

    /// internal semantics of this command history.
    ///
    /// this method gives the "historical semantics" for the given "historical command"
    fn intern_eval(&mut self, state:&mut Self::State, command:&Self::HCommand) -> Result<Self::Resp, Self::Error>;

    /// external semantics of this command history
    ///
    /// this method records a state-transition triple in its representation, which
    /// happens when the environment changes the editor state.
    ///
    /// Q: What assumptions can we make about the state triples?
    ///    e.g., do we ever miss state changes, or do they always sequence entirely?
    ///
    /// provisionally: the sequence of triples is always "well
    /// formed", where each "after" state becomes the "before" state
    /// in the subsequent triple, if any.
    ///
    fn extern_triple(&mut self, before:&Self::State, command:&Self::Command, after:&Self::State);
}

/// store and explore a history of all issued commands
pub mod command_history {
    use std::rc::Rc;
    use super::{Command, Nat};

    pub type Commands = Vec<Command>;

    /// A History organizes related command sequences over time.
    ///
    /// Fork is most general, then Linear, and finally, there's Empty.
    ///
    /// A fork can be introduced in two ways:
    ///  - Undo + Redo
    ///  -
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
