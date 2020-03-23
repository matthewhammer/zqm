/// The ZQM language: abstract syntax
pub mod lang {
    use crate::{bitmap, candid, chain, grid, menu};
    use hashcons::merkle::Merkle;
    use serde::{Deserialize, Serialize};

    /// Media combines words and images
    /// (eventually, we add sound and moving images)
    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub enum Media {
        Void,
        Atom(Atom),
        Name(Name),
        Location(Location),
        MenuTree(Box<menu::MenuTree>),
        Bitmap(Box<bitmap::Bitmap>),
        Chain(Box<chain::Chain>),
        Grid(Box<grid::Grid>),
        Store(Store),
        StoreProj(Store, Name),
        Named(Name, Box<Media>),
        Located(Location, Box<Media>),
        Merkle(Merkle<Media>),
        /// quoted code is media
        Quote(Box<Exp>),
    }

    /*
    Possible idea:
    Introduce LISP-like quotes in a type-theoretic way,
    a la a dual-context, modal interpretation of quoting;
    see Georgios Alexandros Kavvos's arxiv paper on
    "intensionality and intensional recursion, and the Godel-Lob axiom", 2017.
    */

    /// We lift Media to an expression language, with media operations, and adapton operations
    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub enum Exp {
        //----------------------------------------------------------------
        // Media forms (think "data values" in the PL sense):
        //----------------------------------------------------------------
        Void,
        Atom(Atom),
        Name(Name),
        Location(Location),
        Bitmap(Box<bitmap::Bitmap>),
        Chain(Box<chain::Chain>),
        Grid(Box<grid::Grid>),
        Store(Store),
        StoreProj(Box<Exp>, Name),
        Named(Name, Box<Exp>),
        Located(Location, Box<Exp>),
        Merkle(Merkle<Exp>),
        /// Quoted code is media
        Quote(Box<Exp>),
        /// Intensional recursion
        FixQuote(Name, Box<Exp>),
        //----------------------------------------------------------------
        // Expression forms (whose evaluation produces Media):
        //----------------------------------------------------------------
        MerkleFrom(Box<Exp>),
        StoreFrom(Name, Box<Exp>),
        Command(Command),
        Block(Block),
        Var(Name),
        //----------------------------------------------------------------
        // Adapton primitives (for the "demanded computation graph", DCG):
        //----------------------------------------------------------------
        Put(Name, Box<Exp>),
        Thunk(Name, Vec<(Name, Exp)>),
        Get(Box<Exp>),
    }

    /// The data result produced by running a command
    pub type Result = std::result::Result<Media, Error>;

    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub enum Error {}

    /// an expression block consists of a sequence of bindings
    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub struct Block {
        pub bindings: Vec<(Name, Exp)>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub struct Store {
        //pub name: Rc<Name>,
        pub name: Merkle<Name>,
        // finite map from names to StoreRecords
        // will be shared, non-linearly, by each associated StoreProj
        // representation to use hash-consing for O(1) clones and O(1) serialize
        pub table: Vec<(Merkle<Name>, Merkle<Media>)>, // todo: use hashcons crate for this
    }

    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub struct StoreRecord {
        //pub name: Rc<Name>,
        //pub content: Rc<Media>,
        pub name: Name,
        pub content: Media,
    }

    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub enum Editor {
        CandidRepl(Box<candid::Repl>),
        Bitmap(Box<bitmap::Editor>),
        Menu(Box<menu::Editor>),
        Chain(Box<chain::Editor>),
        Grid(Box<grid::Editor>),
    }

    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub struct Frame {
        pub name: Name,
        pub editor: Editor,
        pub cont: FrameCont,
    }

    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub struct FrameCont {
        pub var: Name,
        pub commands: Vec<Command>,
    }

    // to do -- eventually, we may want these to be "open" wrt the exp environment;
    // for expressing scripts, etc; then we'd need to do substitution, or more env-passing, or both.
    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
    pub enum Command {
        Return(Media),
        Resume(Media, Frame),
        Menu(menu::Command),
        Bitmap(bitmap::Command),
        Chain(chain::Command),
        Grid(grid::Command),
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
    pub enum Dir1D {
        Forward,
        Backward,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
    pub enum Dir2D {
        Up,
        Down,
        Left,
        Right,
    }

    #[derive(Debug, Serialize, Deserialize, Hash)]
    pub struct State {
        pub stack: Vec<Frame>,
        pub frame: Frame,
    }

    pub type Hash = u64;
    pub type Nat = usize;

    pub type Map<X, Y> = std::collections::HashMap<X, Y>;

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
    pub enum Name {
        Void,
        Atom(Atom),
        TaggedTuple(Box<Name>, Vec<Name>),
        Merkle(Merkle<Name>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
    pub enum Atom {
        Bool(bool),
        Usize(usize),
        String(String),
        // Eventually(as of 2020-01-04): Permit Media to name Media.
    }

    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub struct Location {
        pub time: Name,
        pub place: Name,
    }

    impl Frame {
        pub fn from_editor(editor: Editor) -> Frame {
            Frame {
                name: Name::Void,
                editor: editor,
                cont: FrameCont {
                    var: Name::Void,
                    commands: vec![],
                },
            }
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// See design/AdaptonDesign.md for details.

pub mod adapton {
    use super::lang::{Exp, Media, Name, Result as EvalResult};
    use crate::adapton;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
    /// media-naming environments
    pub struct Env {
        pub bindings: Vec<(Name, Media)>,
    }
    /// media-producing closures
    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
    pub struct Closure {
        pub env: Env,
        pub exp: Exp,
    }
    /// a Ref node names a "locus of changing data" within the DCG;
    /// when observed by a thunk node, it records this dependent as a new `incoming` edge.
    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
    pub struct Ref {
        pub content: Media,
        // aka, dependents that depend on this node
        pub incoming: Vec<Edge>,
    }
    /// a Thunk node defines a "locus of changing demand & control" within the DCG;
    /// when observed, it performs actions on other nodes,
    /// each recorded as an `outgoing` edge on its dependency, another node.
    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
    pub struct Thunk {
        pub closure: Closure,
        pub result: Option<EvalResult>,
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
    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
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
    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
    pub enum Action {
        /// allocate/overwrite a ref node with given media
        Put(Media),
        /// allocate/overwrite a thunk node with the given media-producing closure
        Thunk(Closure),
        /// demand/observe the media content of a ref/thunk node
        Get(Media),
    }
    /// The public type exposed by ref and thunk allocation
    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
    pub struct NodeId {
        pub name: Name,
    }
    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
    pub enum Node {
        Ref(Ref),
        Thunk(Thunk),
    }
    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
    pub enum Agent {
        Archivist,
        Editor,
    }
    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
    pub struct Store(pub Vec<(Name, Node)>);
    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
    pub struct Stack(pub Vec<Name>);
    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
    pub struct Context {
        pub agent: Agent,
        pub edges: Vec<Edge>,
        pub stack: Stack,
        pub store: Store,
        pub log_buf: LogEvents,
        pub log_stack: Vec<LogEvents>,
    }
    impl Context {
        pub fn enter_scope(&mut self, name: Name) {
            adapton::enter_scope(self, name)
        }
        pub fn leave_scope(&mut self) {
            adapton::leave_scope(self)
        }
        pub fn put_thunk(
            &mut self,
            name: Option<Name>,
            closure: Closure,
        ) -> Result<NodeId, adapton::PutError> {
            adapton::put_thunk(self, name, closure)
        }
        pub fn put(&mut self, name: Name, media: Media) -> Result<NodeId, adapton::PutError> {
            adapton::put(self, name, media)
        }
        pub fn get(&mut self, name: Name, node: NodeId) -> Result<EvalResult, adapton::GetError> {
            adapton::get(self, name, node)
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
    pub enum LogEvent {
        Put(Name, Media, LogEvents),
        PutThunk(Name, Closure, LogEvents),
        Get(Name, Closure, LogEvents),
        DirtyIncomingTo(Name, LogEvents),
        CleanEdgeTo(Name, bool, LogEvents),
        CleanThunk(Name, bool, LogEvents),
        EvalThunk(Name, EvalResult, LogEvents),
    }
    pub enum LogEventTag {
        Put(Name, Media),
        PutThunk(Name, Closure),
        Get(Name, Closure),
        DirtyIncomingTo(Name),
        CleanEdgeTo(Name, bool),
        CleanThunk(Name, bool),
        EvalThunk(Name, EvalResult),
    }
    pub type LogEvents = Vec<LogEvent>;
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// system input
pub mod event {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub enum Event {
        Quit,
        KeyDown(KeyEventInfo),
        KeyUp(KeyEventInfo),
    }
    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub struct KeyEventInfo {
        pub key: String,
        pub alt: bool,
        pub ctrl: bool,
        pub meta: bool,
        pub shift: bool,
    }
}

/// system output
pub mod render {
    use super::lang::Name;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub enum Color {
        RGB(usize, usize, usize),
    }
    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub struct Dim {
        pub width: usize,
        pub height: usize,
    }
    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub struct Pos {
        pub x: isize,
        pub y: isize,
    }
    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub struct Rect {
        pub pos: Pos,
        pub dim: Dim,
    }
    impl Rect {
        pub fn new(x: isize, y: isize, w: usize, h: usize) -> Rect {
            Rect {
                pos: Pos { x, y },
                dim: Dim {
                    width: w,
                    height: h,
                },
            }
        }
    }
    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub struct Node {
        pub name: Name,
        pub rect: Rect,
        pub fill: Fill,
        pub children: Elms,
    }
    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub enum Fill {
        Open(Color, usize), // border width
        Closed(Color),
        None,
    }
    #[derive(Clone, Debug, Serialize, Deserialize, Hash)]
    pub enum Elm {
        Rect(Rect, Fill),
        Node(Box<Node>),
    }
    pub type Elms = Vec<Elm>;
}

/// Deprecated?
pub mod util {
    use super::lang::{Atom, Name};

    /*
    pub fn namefn_id() -> NameFn {
        NameFn{path:vec![]}
    }
     */

    pub fn name_of_str(s: &str) -> Name {
        let atom = Atom::String(s.to_string());
        Name::Atom(atom)
    }

    pub fn name_of_usize(u: usize) -> Name {
        let atom = Atom::Usize(u);
        Name::Atom(atom)
    }

    pub fn name_of_string(s: String) -> Name {
        let atom = Atom::String(s);
        Name::Atom(atom)
    }
}
