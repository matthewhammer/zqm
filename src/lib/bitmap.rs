// Serde: Persistent state between invocations of ZQM
use serde::{Deserialize, Serialize};

use types::{Nat, Dir2D};

/// a grid of bits, represented as a 2D array
#[derive(Debug, Serialize, Deserialize)]
pub struct Bitmap {
    pub width: Nat,
    pub height: Nat,
    pub major: Major,
    pub bits: Vec<Vec<bool>>,
}

/// row-versus-column major order for grid representation
#[derive(Debug, Serialize, Deserialize)]
pub enum Major {
    /// row major ordering (rows indexed first, then columns)
    Row,

    /// column major ordering (columns indexed first, then rows)
    Col
}

/// commands that advance the state of the bitmap,
/// whose execution is independent of editor state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutoCommand {
    /// toggle the bit at the given coordinate
    ToggleBit(Nat, Nat),

    /// set the bit at the given coordinate to the given Boolean value
    SetBit(Nat, Nat, bool),
}

/// the (history-independent) state of the editor
#[derive(Debug, Serialize, Deserialize)]
pub struct EditorState {
    /// created by an Init command; affected by Auto and Edit commands
    pub bitmap: Bitmap,

    /// initialized by an Init command; affected by Edit commands (but not Auto commands)
    pub cursor: (Nat, Nat)
}

/// the full (history-dependent) state of the editor
#[derive(Debug, Serialize, Deserialize)]
pub struct Editor {
    /// full linear history of this bitmap's evolution, as a sequence of commands
    pub history: Vec<Command>,

    /// current state of the bitmap and surrounding editor environment
    pub state: Option<EditorState>,
}

/// commands that create new bitmaps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InitCommand {
    /// make a new 8x8 grid of bits
    Make8x8,

    /// make a new 16x16 grid of bits
    Make16x16,

    /// make a new 32x32 grid of bits
    Make32x32,
}

/// commands that advance the editor state,
/// and possibly, its associated bitmap state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditCommand {
    /// move the grid cursor one unit in a relative direction
    MoveRel(Dir2D),

    /// move the grid cursor to a absolute position
    MoveAbs(Nat, Nat),

    /// toggle the bit at the cursor's grid position
    Toggle,
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

    /// User requests an interactive editor from which to issue
    /// further commands
    Editor,
}

pub mod io {
    use super::{EditorState, EditCommand, Dir2D};
    use sdl2::event::Event;

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
                match kc {
                    Keycode::Space => Ok(vec![EditCommand::Toggle]),
                    Keycode::Left  => Ok(vec![EditCommand::MoveRel(Dir2D::Left)]),
                    Keycode::Right => Ok(vec![EditCommand::MoveRel(Dir2D::Right)]),
                    Keycode::Up    => Ok(vec![EditCommand::MoveRel(Dir2D::Up)]),
                    Keycode::Down  => Ok(vec![EditCommand::MoveRel(Dir2D::Down)]),
                    _              => Ok(vec![]),
                }
            },
            _ => {
                Ok(vec![])
            }
        }
    }

    use sdl2::render::{Canvas, RenderTarget};
    pub fn produce_output<T: RenderTarget>(
        canvas: &mut Canvas<T>,
        edit_state: &EditorState,
    ) -> Result<(), String>
    {
        use sdl2::rect::{Rect};
        use sdl2::pixels::Color;

        let zoom = 64u32;
        let width = 8u32;
        let height = 8u32;
        let border_width = 2u32;

        let cursor_rect = Rect::new(
            edit_state.cursor.0 as i32 * zoom as i32 - border_width as i32,
            edit_state.cursor.1 as i32 * zoom as i32 - border_width as i32,
            zoom + border_width * 2,
            zoom + border_width * 2,
        );

        // grid border is a single background rect:
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.fill_rect(
            Rect::new(
                0,
                0,
                width * zoom + border_width,
                height * zoom + border_width,
            )
        )?;
        canvas.set_draw_color(Color::RGB(150, 255, 150));
        canvas.fill_rect(cursor_rect)?;
        // grid cells are rects:
        for x in 0i32..width as i32 {
            for y in 0i32..height as i32 {
                let cell_rect =
                    Rect::new(
                        x * zoom as i32 + border_width as i32,
                        y * zoom as i32 + border_width as i32,
                        zoom            - border_width * 2,
                        zoom            - border_width * 2,
                    );
                let bit = super::semantics::bitmap_get_bit(
                    &edit_state.bitmap, x as usize, y as usize
                );
                let border_color = Color::RGB(0, 0, 0);
                let cell_color =
                    match (bit, (x as usize, y as usize) == edit_state.cursor) {
                        | (false, false) => Color::RGB(255, 255, 255),
                        | (false, true)  => Color::RGB(240, 255, 240),
                        | (true,  false) => Color::RGB(0, 0, 0),
                        | (true,  true)  => Color::RGB(0, 100, 0),
                    };
                canvas.set_draw_color(cell_color);
                canvas.fill_rect(cell_rect)?;
                canvas.set_draw_color(border_color);
                canvas.draw_rect(cell_rect)?;
            }
        }
        canvas.present();
        Ok(())
    }
}

/// semantic definitions for bitmaps and bitmap editors.
///
/// the mathematical semantics of bitmaps and bitmap editing, in
/// terms of the bitmap representation above,
/// and the abstract syntax of its associated commands
/// (independent from any IO library implementation details).
pub mod semantics {
    use super::{Bitmap, Major, AutoCommand};
    use super::{Editor, EditorState, Command, InitCommand, EditCommand, Dir2D};

    fn bitmap_init(w:usize, h:usize) -> Bitmap {
        let row = vec![ false ; w ];
        let bits = vec![ row.clone() ; h ];
        Bitmap {
            width: row.len(),
            height: bits.len(),
            major: Major::Row,
            bits: bits,
        }
    }

    pub fn bitmap_set_bit(bitmap:&mut Bitmap, x:usize, y:usize, b:bool) {
        match bitmap.major {
            Major::Row => bitmap.bits[x][y] = b,
            Major::Col => bitmap.bits[y][x] = b,
        };
        trace!("bitmap_set_bit({}, {}, {})", x, y, b);
    }

    pub fn bitmap_get_bit(bitmap:&Bitmap, x:usize, y:usize) -> bool {
        let b = match bitmap.major {
            Major::Row => bitmap.bits[x][y],
            Major::Col => bitmap.bits[y][x],
        };
        trace!("bitmap_get_bit({}, {}) = {}", x, y, b);
        b
    }

    pub fn bitmap_toggle_bit(bitmap:&mut Bitmap, x:usize, y:usize) -> bool {
        let b = bitmap_get_bit(bitmap, x, y);
        bitmap_set_bit(bitmap, x, y, !b);
        trace!("bitmap_toggle_bit({}, {}) = {}", x, y, !b);
        !b
    }

    pub fn bitmap_eval(bitmap:&mut Bitmap, command:&AutoCommand) -> Result<(), String> {
        debug!("bitmap_eval: {:?}", command);
        match command {
            &AutoCommand::ToggleBit(x, y) => { bitmap_toggle_bit(bitmap, x, y); },
            &AutoCommand::SetBit(x, y, b) => { bitmap_set_bit(bitmap, x, y, b); },
        };
        Ok(())
    }

    pub fn editor_state_eval(editor:&mut EditorState,
                             command:&EditCommand) -> Result<(), String>
    {
        debug!("editor_state_eval: {:?}", command);
        match command {
            &EditCommand::MoveRel(ref dir) => {
                let (w, h) = (editor.bitmap.width, editor.bitmap.height);
                let (x, y) = editor.cursor;
                editor.cursor = match *dir {
                    Dir2D::Left  => (if x == 0 { 0 } else { x - 1 }, y),
                    Dir2D::Right => (if x + 1 >= w { w - 1 } else { x + 1 }, y),
                    Dir2D::Up    => (x, if y == 0 { 0 } else { y - 1 }),
                    Dir2D::Down  => (x, if y + 1 >= h { h - 1 } else { y + 1 }),
                };
                Ok(())
            }
            &EditCommand::MoveAbs(x, y) => {
                if x < editor.bitmap.width &&
                    y < editor.bitmap.height {
                        editor.cursor = (x, y);
                        Ok(())
                    }
                else {
                    Err("MoveAbs: invalid coordinate".to_string())
                }
            }
            &EditCommand::Toggle => {
                let (x, y) = editor.cursor;
                let _ = bitmap_toggle_bit(&mut editor.bitmap, x, y);
                Ok(())
            }
        }
    }

    pub fn editor_eval(editor:&mut Editor, command:&Command) -> Result<(), String> {
        debug!("editor_eval(#{}): {:?}", editor.history.len(), command);
        // save the command in the history
        editor.history.push( command.clone() );
        // evaluate the command in the appropriate evaluation context:
        match command {
            &Command::Init(ref command) => {
                editor.state = Some(EditorState{
                    bitmap: match command {
                        &InitCommand::Make8x8   => bitmap_init(8,  8),
                        &InitCommand::Make16x16 => bitmap_init(16, 16),
                        &InitCommand::Make32x32 => bitmap_init(32, 32),
                    },
                    cursor: (0, 0),
                });
                Ok(())
            }
            &Command::Auto(ref command) => {
                match editor.state {
                    None => Err("Invalid editor state".to_string()),
                    Some(ref mut st) => bitmap_eval(&mut st.bitmap, &command),
                }
            }
            &Command::Edit(ref command) => {
                match editor.state {
                    None => Err("Invalid editor state".to_string()),
                    Some(ref mut st) => editor_state_eval(st, &command),
                }
            }
            &Command::Editor => {
                // Test if the editor state is initialized; Err if not; Ok if so.
                match editor.state {
                    None => Err("Invalid editor state".to_string()),
                    Some(ref mut _st) => Ok(()),
                }
            }
        }
    }
}
