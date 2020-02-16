// Serde: Persistent state between invocations of ZQM
use serde::{Deserialize, Serialize};
use types::lang::{Dir1D, Media, Name};

/// a chain is an affine linked-list of nodes, each with optionally-named media.
#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum Chain {
    Empty,
    Node(Node, Box<Chain>),
}

/// a node contains media, with an optional name.
#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Node {
    pub name: Name,
    pub media: Box<Media>,
}

/// errors that may arise from chain methods, and `AutoCommand` evaluation.
#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum AutoError {
    /// error for insert/delete/replace when name is absent
    AbsentName(Name, Option<Media>),
    /// error for delete command when media is absent
    AbsentMedia(Option<Name>),
}

pub type Res<R> = Result<R, AutoError>;
pub type Unit = Res<()>;

impl Chain {
    pub fn insert_start(&mut self, name: Name, media: Media) -> Unit {
        unimplemented!()
    }
    pub fn insert_end(&mut self, name: Name, media: Media) -> Unit {
        unimplemented!()
    }
    pub fn insert_after(&mut self, name: Name, name_new: Name, media: Media) -> Unit {
        unimplemented!()
    }
    pub fn insert_before(&mut self, name: Name, name_new: Name, media: Media) -> Unit {
        unimplemented!()
    }

    pub fn delete_start(&mut self) -> Res<Media> {
        unimplemented!()
    }
    pub fn delete_end(&mut self) -> Res<Media> {
        unimplemented!()
    }
    pub fn delete_after(&mut self, name: Name) -> Res<Media> {
        unimplemented!()
    }
    pub fn delete_before(&mut self, name: Name) -> Res<Media> {
        unimplemented!()
    }

    pub fn replace_start(&mut self, media: Media) -> Res<Media> {
        unimplemented!()
    }
    pub fn replace_end(&mut self, media: Media) -> Res<Media> {
        unimplemented!()
    }
    pub fn replace(&mut self, name: Name, media: Media) -> Res<Media> {
        unimplemented!()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum AutoCommand {
    InsertStart(Name, Media),
    InsertEnd(Name, Media),
    InsertBefore(Name, Name, Media),
    InsertAfter(Name, Name, Media),

    DeleteStart,
    DeleteEnd,
    DeleteAfter(Name),
    DeleteBefore(Name),

    Replace(Name, Media),
    ReplaceEnd(Media),
    ReplaceStart(Media),
}

use self::AutoCommand::*;

impl AutoCommand {
    pub fn is_insert(&self) -> bool {
        match &self {
            &InsertStart(_, _) => true,
            &InsertEnd(_, _) => true,
            &InsertAfter(_, _, _) => true,
            &InsertBefore(_, _, _) => true,
            _ => false,
        }
    }
    pub fn is_delete(&self) -> bool {
        match &self {
            &DeleteStart => true,
            &DeleteEnd => true,
            &DeleteAfter(_) => true,
            &DeleteBefore(_) => true,
            _ => false,
        }
    }
    pub fn is_replace(&self) -> bool {
        match &self {
            &Replace(_, _) => true,
            &ReplaceStart(_) => true,
            &ReplaceEnd(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct EditorState {
    pub head: Chain,
    pub cursor: Option<Node>,
    pub tail: Chain,
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct Editor {
    pub history: Vec<Command>,
    pub state: Option<EditorState>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum InitCommand {
    Empty,
    String(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum EditCommand {
    MoveRel(Dir1D),
    MoveAbs(usize),
    MoveBegin,
    MoveEnd,
    Insert(Dir1D, Name, Media),
    Delete(Dir1D),
    Replace(Media),
}

/// commands that advance the evolution of a bitmap
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum Command {
    /// commands that create new bitmaps
    Init(InitCommand),

    /// commands that advance the state of the bitmap,
    /// whose execution is independent of editor state, once we select a "direction".
    Auto(Dir1D, AutoCommand),

    /// commands that advance the editor state,
    /// and possibly, its associated bitmap state
    Edit(EditCommand),
}

pub mod semantics {
    //use super::{Chain, Command, AutoCommand, EditCommand, InitCommand, Editor, EditorState};
    use super::{AutoCommand, Chain, Command, EditCommand, EditorState, Media, Res};
    use glyph::cap5x5::glyph_map;

    // todo -- if we instead assume a moved Command rather than a borrowed one, we avoid clone()s here?
    //         OTOH, if we use a borrow, the Command constructors are affine too, which can be annoying, esp for logging.

    pub fn chain_eval(chain: &mut Chain, command: &AutoCommand) -> Res<Option<Media>> {
        trace!("chain_eval {:?} ...", command);
        use self::AutoCommand::*;
        pub fn some(r: Res<Media>) -> Res<Option<Media>> {
            r.map(|m| Some(m))
        };
        pub fn none(r: Res<()>) -> Res<Option<Media>> {
            r.map(|_| None)
        };
        let _gm = glyph_map();
        let res = match &command {
            InsertStart(ref n, ref m) => none(chain.insert_start(n.clone(), m.clone())),
            DeleteStart => some(chain.delete_start()),
            InsertEnd(ref n, ref m) => none(chain.insert_end(n.clone(), m.clone())),
            DeleteEnd => some(chain.delete_end()),
            Replace(ref n, ref m) => some(chain.replace(n.clone(), m.clone())),
            ReplaceStart(ref m) => some(chain.replace_start(m.clone())),
            ReplaceEnd(ref m) => some(chain.replace_end(m.clone())),
            InsertAfter(ref n, ref nn, ref m) => {
                none(chain.insert_after(n.clone(), nn.clone(), m.clone()))
            }
            DeleteAfter(ref n) => some(chain.delete_after(n.clone())),
            InsertBefore(ref n, ref nn, ref m) => {
                none(chain.insert_before(n.clone(), nn.clone(), m.clone()))
            }
            DeleteBefore(ref n) => some(chain.delete_before(n.clone())),
        };
        trace!("chain_eval {:?} ==> {:?}", command, res);
        res
    }

    pub fn editor_state_eval(
        editor: &mut EditorState,
        command: &EditCommand,
    ) -> Result<(), String> {
        trace!("editor_state_eval: {:?}", command);
        unimplemented!()
    }

    pub fn editor_eval(editor: &mut EditorState, command: &Command) -> Result<(), String> {
        trace!("editor_eval: {:?}", command);
        unimplemented!()
    }
}

/*
pub mod io {
    //use super::{EditorState, EditCommand, Dir1D};
    use super::{EditCommand, EditorState};
    use sdl2::event::Event;
    use types::render;

    pub fn consume_input(event: Event) -> Result<Vec<EditCommand>, ()> {
        use sdl2::keyboard::Keycode;
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => Err(()),
            Event::KeyDown {
                keycode: Some(kc), ..
            } => {
                // todo
                match kc {
                    Keycode::Space => Ok(vec![]),
                    Keycode::Left => Ok(vec![]),
                    Keycode::Right => Ok(vec![]),
                    Keycode::Up => Ok(vec![]),
                    Keycode::Down => Ok(vec![]),
                    _ => Ok(vec![]),
                }
            }
            _ => Ok(vec![]),
        }
    }

    use sdl2::render::{Canvas, RenderTarget};
    pub fn render_elms<T: RenderTarget>(
        canvas: &mut Canvas<T>,
        edit_state: &EditorState,
    ) -> Result<render::Elms, String> {
        let out: render::Elms = vec![];
        //use sdl2::rect::{Rect};
        //use sdl2::pixels::Color;

        // todo
        Ok(out)
    }
}
*/
