// Note: Using Box<_> in places where I might like Rc<_>;
//       Serde does not work for Rc<_> out of the box
//       Same issue for HashMap<_> uses here.

// Serde: Persistent state between invocations of ZQM
use serde::{Deserialize, Serialize};
use std::rc::Rc;

/// Media combines words and images
/// (eventually, we add sound and moving images)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Media {
    Void,
    Atom(Atom),
    Name(Name),
    Location(Location),
    Named(Name, Box<Media>),
    Located(Location, Box<Media>),
    Bitmap(Box<super::bitmap::Bitmap>),
    Chain(Box<super::chain::Chain>),
    Grid(Box<super::grid::Grid>),
    Store(Store),
    StoreProj(Store, Name),
}

/// We lift Media to an expression language, with media operations, and adapton operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Exp {
    Void,
    Atom(Atom),
    Name(Name),
    Location(Location),
    Named(Name, Box<Exp>),
    Located(Location, Box<Exp>),
    Bitmap(Box<super::bitmap::Bitmap>),
    Chain(Box<super::chain::Chain>),
    Grid(Box<super::grid::Grid>),
    Store(Store),
    StoreProj(Box<Exp>, Name),
    //----------------------------------------------------------------
    // Media forms above; More expression forms below:
    //----------------------------------------------------------------
    Block(Block),
    StoreFrom(Name, Box<Exp>),
    Command(Command),
    //----------------------------------------------------------------
    // Adapton primitives (for the "demanded computation graph", DCG):
    //----------------------------------------------------------------
    Put(Name, Box<Exp>),
    Thunk(Name, Vec<(Name, Exp)>),
    Get(adapton::NodeId),
}

/// an expression block consists of a sequence of bindings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub bindings: Vec<(Name, Exp)>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Store {
    //pub name: Rc<Name>,
    pub name: Name,
    // finite map from names to StoreRecords
    // will be shared, non-linearly, by each associated StoreProj
    // representation to use hash-consing for O(1) clones and O(1) serialize

    // todo: use hashcons crate for this
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoreRecord {
    //pub name: Rc<Name>,
    //pub content: Rc<Media>,
    pub name: Name,
    pub content: Media,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Editor {
    Bitmap(Box<super::bitmap::Editor>),
    Chain(Box<super::chain::Editor>),
    Grid(Box<super::grid::Editor>),
}

// to do -- eventually, we may want these to be "open" wrt the exp environment;
// for expressing scripts, etc; then we'd need to do substitution, or more env-passing, or both.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Bitmap(super::bitmap::Command),
    Chain(super::chain::Command),
    Grid(super::grid::Command),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Dir1D {
    Forward,
    Backward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Dir2D {
    Up,
    Down,
    Left,
    Right
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub editor: Editor,
}

pub type Hash = u64;
pub type Nat = usize;

pub type Map<X,Y> = std::collections::HashMap<X,Y>;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Name {
    pub tree: Box<NameTree>,
    // Eventually(as of 2020-01-04): pub hash: Hash,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum NameTree {
    Atom(Atom),
    Option(Option<Name>),
    TaggedTuple(Name, Vec<Name>)
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Atom {
    Bool(bool),
    Usize(usize),
    String(String),
    // Eventually(as of 2020-01-04): Permit Media to name Media.
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Location {
    pub time:  Name,
    pub place: Name,
}



// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// See design/AdaptonDesign.md for details.

pub mod adapton {
    use serde::{Deserialize, Serialize};
    use super::{Name, Media, Command, Exp};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    /// media-naming environments
    pub struct Env {
        pub bindings:Vec<(Name, Media)>
    }
    /// media-producing closures
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Closure {
        pub env: Env,
        pub exp: Exp,
    }
    /// a Ref node names a "locus of changing data" within the DCG;
    /// when observed by a thunk node, it records this dependent as a new `incoming` edge.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Ref {
        pub content: Media,
        // aka, dependents that depend on this node
        pub incoming: Vec<Edge>,
    }
    /// a Thunk node defines a "locus of changing demand & control" within the DCG;
    /// when observed, it performs actions on other nodes, 
    /// each recorded as an `outgoing` edge on its dependency, another node.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Thunk {
        pub closure: Closure,
        pub result: Option<Media>,
        // aka, dependencies upon which this node depends
        pub outgoing: Vec<Edge>,
        // aka, dependents that depend on this node
        pub incoming: Vec<Edge>,
    }
    /// Each edge relates two nodes, the first of which is always a
    /// (demanded) thunk; the edge records a single action performed
    /// by this thunk.  Later, the edge can be dirtied as a by-product of
    /// future actions to the source of the edge, or its transitive dependencies.
    /// Dirty edges cannot be reused via memoization.
    /// Invariant: not(dirty_flag) implies checkpoint is consistent, transitively.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Edge {
        /// edge source; subject of the action along edge
        pub dependent: NodeId,
        /// edge target; object of the action along edge
        pub dependency: NodeId,
        /// save a checkpoint of the action along edge
        pub checkpoint: Action,
        /// not(dirty_flag) implies checkpoint is consistent, transitively
        pub dirty_flag: bool,
    }
    /// The data associated with an action as required by an edge's checkpoint of that action
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Action {
        /// allocate/overwrite a ref node with given media
        Put(Media),
        /// allocate/overwrite a thunk node with the given media-producing closure
        Thunk(Closure),
        /// demand/observe the media content of a ref/thunk node
        Get(Media),
    }
    /// The public type exposed by ref and thunk allocation
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NodeId {
        pub name: Name
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Node {
        Ref(Ref),
        Thunk(Thunk),
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Context {

    }
    impl Context {
        fn enter_scope(name:Name) {
            unimplemented!()
        }
        fn leave_scope() {
            unimplemented!()
        }
        fn thunk(name:Option<Name>, closure:Closure) -> NodeId {
            unimplemented!()
        }
        fn set(name:Option<Name>, media:Media) -> NodeId {
            unimplemented!()
        }
        fn get(name:Name, node:NodeId) {
            unimplemented!()
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

pub mod render {
    use sdl2::pixels::Color;

    pub struct Dim {
        pub width: usize,
        pub height: usize,
    }
    pub struct Pos {
        pub x: usize,
        pub y: usize,
    }
    pub struct Rect {
        pub pos: Pos,
        pub dim: Dim,
    }
    pub struct Node {
        pub rect: Rect,
        pub fill: Fill,
        pub children: Elms,
    }
    pub enum Fill {
        Open(Color, usize), // border width
        Closed(Color),
        None,
    }
    pub enum Elm {
        Rect(Rect, Fill),
        Node(Box<Node>)
    }
    pub type Elms = Vec<Elm>;
}


pub mod util {
    use super::*;

    /*
    pub fn namefn_id() -> NameFn {
        NameFn{path:vec![]}
    }
     */

    pub fn name_of_str(s:&str) -> Name {
        let atom = Atom::String(s.to_string());
        Name{tree:Box::new(NameTree::Atom(atom))}
    }

    pub fn name_of_usize(u:usize) -> Name {
        let atom = Atom::Usize(u);
        Name{tree:Box::new(NameTree::Atom(atom))}
    }

    pub fn name_of_string(s:String) -> Name {
        let atom = Atom::String(s);
        Name{tree:Box::new(NameTree::Atom(atom))}
    }
}


/*

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
*/

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
