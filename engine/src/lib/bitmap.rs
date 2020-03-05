// Serde: Persistent state between invocations of ZQM
use serde::{Deserialize, Serialize};

use types::lang::{Dir2D, Nat};

// Step 1:
// -------
// Define the structure, in terms of "simplified, affine Rust"
// (no references or lifetimes; everything is affine, so no Rc<_>s either.)

/// a grid of bits, represented as a 2D array
#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Bitmap {
    pub width: Nat,
    pub height: Nat,
    pub major: Major,
    pub bits: Vec<Vec<bool>>,
}

/// row-versus-column major order for grid representation
#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum Major {
    /// row major ordering (rows indexed first, then columns)
    Row,

    /// column major ordering (columns indexed first, then rows)
    Col,
}

// Step 2:
// -------
// Define the structure's "auto commands", as a DSL datatype.

/// commands that advance the state of the bitmap,
/// whose execution is independent of editor state
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum AutoCommand {
    /// toggle the bit at the given coordinate
    ToggleBit(Nat, Nat),

    /// set the bit at the given coordinate to the given Boolean value
    SetBit(Nat, Nat, bool),
}

// Step 3:
// -------
//
// Define a canonical editor for the structure in question.  Again, use simplified, affine Rust.

/// the history-_independent_ state of the editor
#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct EditorState {
    /// created by an Init command; affected by Auto and Edit commands
    pub bitmap: Bitmap,

    /// initialized by an Init command; affected by Edit commands (but not Auto commands)
    pub cursor: (Nat, Nat),
}

// Step (3b) --
// The `Editor` definition
//   includes the command history, and any "pre-states" before initialization completes.

/// the history-_dependent_ state of the editoro
#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct Editor {
    /// full linear history of this bitmap's evolution, as a sequence of commands
    pub history: Vec<Command>,

    /// current state of the bitmap and surrounding editor environment
    pub state: Option<EditorState>,
}

// Step 4a:
// -------
// Define commands that initialize the editor state.

/// commands that create new bitmaps
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum InitCommand {
    /// make a new 8x8 grid of bits
    Make8x8,

    /// make a new 16x16 grid of bits
    Make16x16,

    /// make a new 32x32 grid of bits
    Make32x32,
}

// Step 4b:
// -------
// Define commands that evolve the editor state with edits,
//   or changes to the edit state (cursor location).

/// commands that advance the editor state,
/// and possibly, its associated bitmap state.
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum EditCommand {
    /// move the grid cursor one unit in a relative direction
    MoveRel(Dir2D),

    /// move the grid cursor to a absolute position
    MoveAbs(Nat, Nat),

    /// toggle the bit at the cursor's grid position
    Toggle,
}

// Step 4c:
// -------
//
// Define a combined language of commands that includes (distinct) Init, Auto
// and Edit sublanguages.

/// commands that advance the evolution of a bitmap
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
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

// Step 5:
// -------
//
// Define the state-change semantics for the command languages.

/// semantic definitions for bitmaps and bitmap editors.
///
/// the mathematical semantics of bitmaps and bitmap editing, in
/// terms of the bitmap representation above,
/// and the abstract syntax of its associated commands
/// (independent from any IO library implementation details).
pub mod semantics {
    use super::{AutoCommand, Bitmap, Major};
    use super::{Command, Dir2D, EditCommand, Editor, EditorState, InitCommand};

    fn bitmap_init(w: usize, h: usize) -> Bitmap {
        let row = vec![false; w];
        let bits = vec![row.clone(); h];
        Bitmap {
            width: row.len(),
            height: bits.len(),
            major: Major::Row,
            bits: bits,
        }
    }

    pub fn bitmap_set_bit(bitmap: &mut Bitmap, x: usize, y: usize, b: bool) {
        match bitmap.major {
            Major::Row => bitmap.bits[y][x] = b,
            Major::Col => bitmap.bits[x][y] = b,
        };
        debug!("bitmap_set_bit({}, {}, {})", x, y, b);
    }

    pub fn bitmap_get_bit(bitmap: &Bitmap, x: usize, y: usize) -> bool {
        let b = match bitmap.major {
            Major::Row => bitmap.bits[y][x],
            Major::Col => bitmap.bits[x][y],
        };
        debug!("bitmap_get_bit({}, {}) ==> {}", x, y, b);
        b
    }

    pub fn bitmap_get_size(bitmap: &Bitmap) -> (usize, usize) {
        (bitmap.width, bitmap.height)
    }

    pub fn bitmap_toggle_bit(bitmap: &mut Bitmap, x: usize, y: usize) -> bool {
        let b = bitmap_get_bit(bitmap, x, y);
        bitmap_set_bit(bitmap, x, y, !b);
        debug!("bitmap_toggle_bit({}, {}) ==> {}", x, y, !b);
        !b
    }

    // a few ideas
    // 1. create an Eval trait, parameterized by specific media and command types.
    // 2. each of these functions is also an impl of this trait for some pair of types.
    // 3. The Result<_,_> types could be richer.
    // 4. The successful return value could be `Media` (the full adjunction), or another trait param.
    // 5. The failure error code could be a common error type (every possible error), or a trait param.

    pub fn bitmap_eval(bitmap: &mut Bitmap, command: &AutoCommand) -> Result<(), String> {
        debug!("bitmap_eval {:?}", command);
        let res = match command {
            &AutoCommand::ToggleBit(x, y) => {
                bitmap_toggle_bit(bitmap, x, y);
            } // to do: actually return the bit.
            &AutoCommand::SetBit(x, y, b) => {
                bitmap_set_bit(bitmap, x, y, b);
            }
        };
        let res = Ok(res);
        debug!("bitmap_eval {:?} ==> {:?}", command, res);
        res
    }

    pub fn editor_state_eval(
        editor: &mut EditorState,
        command: &EditCommand,
    ) -> Result<(), String> {
        debug!("editor_state_eval {:?}", command);
        let res = match command {
            &EditCommand::MoveRel(ref dir) => {
                let (w, h) = (editor.bitmap.width, editor.bitmap.height);
                let (x, y) = editor.cursor;
                editor.cursor = match *dir {
                    Dir2D::Left => (if x == 0 { 0 } else { x - 1 }, y),
                    Dir2D::Right => (if x + 1 >= w { w - 1 } else { x + 1 }, y),
                    Dir2D::Up => (x, if y == 0 { 0 } else { y - 1 }),
                    Dir2D::Down => (x, if y + 1 >= h { h - 1 } else { y + 1 }),
                };
                Ok(())
            }
            &EditCommand::MoveAbs(x, y) => {
                if x < editor.bitmap.width && y < editor.bitmap.height {
                    editor.cursor = (x, y);
                    Ok(())
                } else {
                    Err("MoveAbs: invalid coordinate".to_string())
                }
            }
            &EditCommand::Toggle => {
                let (x, y) = editor.cursor;
                let _ = bitmap_toggle_bit(&mut editor.bitmap, x, y);
                Ok(())
            }
        };
        debug!("editor_state_eval {:?} ==> {:?}", command, res);
        res
    }

    pub fn editor_eval(editor: &mut Editor, command: &Command) -> Result<(), String> {
        let num = editor.history.len();
        debug!("#{}: editor_eval {:?}", num, command);
        // save the command in the history
        editor.history.push(command.clone());
        // evaluate the command in the appropriate evaluation context:
        let res = match command {
            &Command::Init(ref command) => {
                editor.state = Some(EditorState {
                    bitmap: match command {
                        &InitCommand::Make8x8 => bitmap_init(8, 8),
                        &InitCommand::Make16x16 => bitmap_init(16, 16),
                        &InitCommand::Make32x32 => bitmap_init(32, 32),
                    },
                    cursor: (0, 0),
                });
                Ok(())
            }
            &Command::Auto(ref command) => match editor.state {
                None => Err("Invalid editor state".to_string()),
                Some(ref mut st) => bitmap_eval(&mut st.bitmap, &command),
            },
            &Command::Edit(ref command) => match editor.state {
                None => Err("Invalid editor state".to_string()),
                Some(ref mut st) => editor_state_eval(st, &command),
            },
        };
        info!("#{}: editor_eval {:?} ==> {:?}", num, command, res);
        res
    }
}

// Step 6:
// -------
//
// Define the IO for the Editor using the abstract `render` module, and associated types.

pub mod io {
    use super::{Dir2D, EditCommand, EditorState};
    use types::event::Event;
    use types::render::{self, Color, Fill, Rect};

    pub fn edit_commands_of_event(event: &Event) -> Result<Vec<EditCommand>, ()> {
        match event {
            &Event::Quit { .. } => Err(()),
            &Event::KeyDown(ref kei) => match kei.key.as_str() {
                "Escape" => Err(()),
                " " => Ok(vec![EditCommand::Toggle]),
                "ArrowLeft" => Ok(vec![EditCommand::MoveRel(Dir2D::Left)]),
                "ArrowRight" => Ok(vec![EditCommand::MoveRel(Dir2D::Right)]),
                "ArrowUp" => Ok(vec![EditCommand::MoveRel(Dir2D::Up)]),
                "ArrowDown" => Ok(vec![EditCommand::MoveRel(Dir2D::Down)]),
                _ => Ok(vec![]),
            },
            _ => Ok(vec![]),
        }
    }

    //use sdl2::render::{Canvas, RenderTarget};
    pub fn render_elms(edit_state: &EditorState) -> Result<render::Elms, String> {
        use render::Render;

        let mut render: Render = Render::new();

        let (width, height) = super::semantics::bitmap_get_size(&edit_state.bitmap);

        // to do -- get these constants from the editor state
        let zoom = 32 as usize;
        let border_width = 2 as usize;
        let cell_width = zoom + border_width * 2;

        let grid_border_color = Color::RGB(100, 80, 100);
        let cursor_border_color = Color::RGB(150, 255, 150);

        fn get_cell_color(is_set: bool, is_focus: bool) -> Color {
            // to do -- get these constants from the editor state
            // cell colors, based on two bits:
            let color_notset_notfocus = Color::RGB(0, 0, 0);
            let color_notset_isfocus = Color::RGB(0, 100, 0);
            let color_isset_notfocus = Color::RGB(255, 225, 255);
            let color_isset_isfocus = Color::RGB(240, 250, 240);
            match (is_set, is_focus) {
                (false, false) => color_notset_notfocus,
                (false, true) => color_notset_isfocus,
                (true, false) => color_isset_notfocus,
                (true, true) => color_isset_isfocus,
            }
        };

        let cursor_rect = Rect::new(
            (edit_state.cursor.0 * cell_width) as isize,
            (edit_state.cursor.1 * cell_width) as isize,
            cell_width,
            cell_width,
        );

        // grid border is a single background rect:
        let grid_rect = Rect::new(0, 0, width * cell_width, height * cell_width);
        render.rect(&grid_rect, Fill::Closed(grid_border_color.clone()));
        render.rect(&cursor_rect, Fill::Closed(cursor_border_color.clone()));

        // grid cells are rects:
        for x in 0..width {
            for y in 0..height {
                let cell_rect = Rect::new(
                    (x * cell_width + border_width) as isize,
                    (y * cell_width + border_width) as isize,
                    zoom,
                    zoom,
                );
                let bit =
                    super::semantics::bitmap_get_bit(&edit_state.bitmap, x as usize, y as usize);
                let cell_color = get_cell_color(bit, (x as usize, y as usize) == edit_state.cursor);
                render.rect(&cell_rect, Fill::Closed(cell_color.clone()));
                render.rect(&cell_rect, Fill::Open(grid_border_color.clone(), 1));
            }
        }
        Ok(render.into_elms())
    }
}
