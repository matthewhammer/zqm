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
    Replace(Name, Media),
    InsertAfter(Name, Media),
    DeleteAfter(Name),
    InsertBefore(Name, Media),
    DeleteBefore(Name),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorState {
    pub head: Chain,
    pub cursor: Option<Node>,
    pub tail: Chain,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Editor {
    pub history: Vec<Command>,
    pub state: Option<EditorState>,
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

mod semantics {
    use super::{Chain, Command, AutoCommand, EditCommand, InitCommand, Editor, EditorState};

    pub fn chain_eval(chain:&mut Chain, command: &AutoCommand) -> Result<(), String> {
        trace!("chain_eval: {:?}", command);
        unimplemented!()
    }

    pub fn editor_state_eval(editor:&mut EditorState,
                             command:&EditCommand) -> Result<(), String>
    {
        trace!("editor_state_eval: {:?}", command);
        unimplemented!()
    }

    pub fn editor_eval(editor:&mut EditorState,
                       command:&Command) -> Result<(), String>
    {
        trace!("editor_eval: {:?}", command);
        unimplemented!()
    }


}

pub mod io {
    use super::{EditorState, EditCommand, Dir1D};
    use sdl2::event::Event;
    use types::render;

    pub fn consume_input(event:Event) -> Result<Vec<EditCommand>, ()> {
        use sdl2::keyboard::Keycode;
        match event {
            Event::Quit {..}
            | Event::KeyDown {
                keycode: Some(Keycode::Escape), ..
            } => {
                Err(())
            },
            Event::KeyDown{keycode:Some(kc), ..} => {
                // todo
                match kc {
                    Keycode::Space => Ok(vec![]),
                    Keycode::Left  => Ok(vec![]),
                    Keycode::Right => Ok(vec![]),
                    Keycode::Up    => Ok(vec![]),
                    Keycode::Down  => Ok(vec![]),
                    _              => Ok(vec![]),
                }
            },
            _ => {
                Ok(vec![])
            }
        }
    }

    use sdl2::render::{Canvas, RenderTarget};
    pub fn render_elms<T: RenderTarget>(
        canvas: &mut Canvas<T>,
        edit_state: &EditorState,
    ) -> Result<render::Elms, String>
    {
        let mut out : render::Elms = vec![];
        use sdl2::rect::{Rect};
        use sdl2::pixels::Color;

        // todo
        Ok(vec![])
    }

}
