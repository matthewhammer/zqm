
// Serde: Persistent state between invocations of ZQM
use serde::{Deserialize, Serialize};

pub type Hash = u64;
pub type Nat = usize;

pub type Map<X,Y> = std::collections::HashMap<X,Y>;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Name {
    pub tree: Box<NameTree>,
    //pub hash: Hash,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum NameTree {
    Atom(NameAtom),
    Option(Option<Name>),
    TaggedTuple(Name, Vec<Name>)
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum NameAtom {
    Bool(bool),
    Usize(usize),
    String(String),
}

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
    BeAtTime(Name),
    BeginTime(Name),
    EndTime,

    MakePlace(Name),
    GoToPlace(Name),
    BeginPlace(Name),
    EndPlace,

    Bitmap(super::bitmap::Command),

    // save/restore manages editor state via an ambient store of named editor states
    Save(Name),
    Restore(Name),

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


/// a state conists of a locus and a tuple of editor states, across the zqm modules.
///
/// the state tuple has an editor component per possible (concurrent, independent) editor type
/// (but for now, we just edit bitmaps)
#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub locus: Locus,
    pub bitmap_editor: super::bitmap::Editor,
}

/// a state store maps state names to states
#[derive(Debug, Serialize, Deserialize)]
pub struct StateStore( Map<Name, State> );

/// a state store tree cursor
///
/// the "cursor position" is a distinct subtree;
/// together with the tree context, the pair forms a "tree zipper"
/// that represents a cursor in a tree, permitting O(1) local navigation.
#[derive(Debug, Serialize, Deserialize)]
pub struct StateStoreTreeCursor {
    pub ctx: StateStoreTreeCtx,
    pub pos: StateStoreTree,
}

/// historical tree of state stores
///
/// a state store tree organizes a set of meta commands' state stores into a
/// historical tree of concurrent evolution; each "node" is a store of
/// states; each "edge" is a MetaCommand that relates two nodes' state stores.
#[derive(Debug, Serialize, Deserialize)]
pub enum StateStoreTree {
    Empty,
    Singleton(StateStore),
    Linear(StateStore, Vec<LinearStateTrans>, Box<StateStoreTree>),
    Branching(StateStore, Vec<BranchingStateTrans>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StateStoreTreeCtx {
    Linear(StateStore, Vec<LinearStateTrans>, Vec<LinearStateTrans>),
    Branching(StateStore, Vec<BranchingStateTransCtx>),
}

/// a linear state transition consists of a `Command` and target `StateStore`
#[derive(Debug, Serialize, Deserialize)]
pub struct LinearStateTrans {
    command: Command,
    target: StateStore,
}

/// a branching state transition consists of a `Command` and target `StateStoreTree`
///
/// two sister state transitions, each of type `BranchingStateTrans`,
/// arise together when a `Command` is undone, and a distinct `Command` is issued;
/// this pattern can repeat, giving rise to more siblings.
#[derive(Debug, Serialize, Deserialize)]
pub struct BranchingStateTrans {
    command: Command,
    subtree: Box<StateStoreTree>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BranchingStateTransCtx {
    ctx: StateStoreTreeCtx,
    command: Command,
}

pub mod util {
    use super::*;

    pub fn namefn_id() -> NameFn {
        NameFn{path:vec![]}
    }

    pub fn name_of_str(s:&str) -> Name {
        let atom = NameAtom::String(s.to_string());
        Name{tree:Box::new(NameTree::Atom(atom))}
    }   

    pub fn name_of_string(s:String) -> Name {
        let atom = NameAtom::String(s);
        Name{tree:Box::new(NameTree::Atom(atom))}
    }   
}
        
/*

/// A `History` generalizes the relationship between interactive
/// `Command`s and the `State`s that they describe and inter-relate.
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

trait History : Eq + Hash + Debug {
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
    fn intern_eval(&mut self,
                   state: &mut Self::State,
                   command: &Self::HCommand)
                   -> Result<Self::Resp, Self::Error>;

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
    fn extern_eval(&mut self,
                   before: &Self::State,
                   command: &Self::Command,
                   result: Result<Self::Resp, Self::Error>,
                   after: &Self::State);
}
*/

/////////////////////////////////

/*
impl FromStr for Name {
    fn from_str(s:& str) -> Name {
        Name::Atom(NameAtom::String(s.to_string()))
    }
}
*/
