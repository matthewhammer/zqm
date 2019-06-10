#![feature(nll)]

#[macro_use] extern crate log;

pub type Nat = u64;

pub type Name = String; // to do

#[derive(Debug,Clone)]
pub struct NameFn {  // to do
    pub path: Vec<Name>
}

#[derive(Debug)]
pub struct Point {
    pub time:  Name,
    pub place: Name,
}

#[derive(Debug)]
pub struct Space {
    pub time:  NameFn,
    pub place: NameFn,
}

#[derive(Debug)]
pub struct Locus {
    pub point: Point,
    pub space: Space,
}

#[derive(Debug)]
pub enum Command {
    Version,
    Completions(String),
    MakeTime(Name),
    MakePlace(Name),
    GotoPlace(Name),
    ReadLine,
    SdlTest,
    Bitmap(bitmap::Command),
}

#[derive(Debug)]
pub enum Dir2D {
    Up,
    Down,
    Left,
    Right
}

pub mod eval {
    use super::{
        //Name,
        NameFn,
        Point,
        Space,
        Locus,
        Command,
        //Dir2D
    };

    #[derive(Debug)]
    pub struct State {
        pub locus: Locus,
        pub command_history: Vec<Command>,
        pub bitmap_editor: super::bitmap::Editor,
    }

    // to do: take "genesis" arguments:
    // - user's self-symbol abbreviation
    // - current date, time, place, etc
    // - OS filesystem paths for archiving
    pub fn init() -> State {
        let id_nmfn = NameFn{path:vec![]};
        let locus_init = Locus{
            point: Point{
                time:  "now".to_string(),
                place: "here".to_string(),
            },
            space: Space {
                time:  id_nmfn.clone(),
                place: id_nmfn,
            },
        };
        let state_init = State {
            locus: locus_init,
            command_history: vec![],
            bitmap_editor: super::bitmap::Editor {
                state: None,
                history: vec![],
            },
        };
        state_init
    }

    pub fn eval(state: &mut State, command:&Command) {
        debug!("begin: eval({:?}, {:?})", state, command);
        unimplemented!()
    }

}


/// primitive visual images which each consist of a 2D grid of bits
pub mod bitmap {
    use super::{Nat, Dir2D};

    /// a grid of bits, represented as a 2D array
    #[derive(Debug)]
    pub struct Bitmap {
        pub width: Nat,
        pub height: Nat,
        pub major: Major,
        pub bits: Vec<Vec<bool>>,
    }

    /// row-versus-column major order for grid representation
    #[derive(Debug)]
    pub enum Major {
        /// row major ordering (rows indexed first, then columns)
        Row,
        /// column major ordering (columns indexed first, then rows)
        Column
    }

    /// commands that create new bitmaps
    #[derive(Debug)]
    pub enum InitCommand {
        /// make a new 8x8 grid of bits
        Make8x8,
        //Make16x16,
        //Make32x32, // to create mask and other elements for past concept art
    }

    /// commands that advance the state of the bitmap,
    /// whose execution is independent of editor state
    #[derive(Debug)]
    pub enum AutoCommand {
        /// set the bit at the given coordinate
        SetBit(Nat, Nat),
        /// clear the bit at the given coordinate
        ClearBit(Nat, Nat),
    }

    /// the (history-independent) state of the editor
    #[derive(Debug)]
    pub struct EditorState {
        /// created by an Init command; affected by Auto and Edit commands
        pub bitmap: Bitmap,
        /// initialized by an Init command; affected by Edit commands (but not Auto commands)
        pub cursor: (Nat, Nat)
    }

    /// the full (history-dependent) state of the editor
    #[derive(Debug)]
    pub struct Editor {
        /// full linear history of this bitmap's evolution, as a sequence of commands
        pub history: Vec<Command>,
        /// current state of the bitmap and surrounding editor environment
        pub state: Option<EditorState>,
    }

    /// commands that advance the editor state,
    /// and possibly, its associated bitmap state.
    #[derive(Debug)]
    pub enum EditCommand {
        /// move the grid cursor, in four relative directions
        Move(Dir2D),
        /// toggle the bit at the cursor's grid position
        Toggle,
    }

    /// commands that advance the evolution of a bitmap
    #[derive(Debug)]
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
}
