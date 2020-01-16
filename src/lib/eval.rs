use super::types::{
    State,
    Editor,
    Command,
    render,
};

use sdl2::event::Event;
pub fn consume_input(state: &mut State, event:Event) -> Result<Vec<Command>, ()> {
    debug!("{:?}", event);
    match &mut state.editor {
        &mut Editor::Bitmap(ref ed) => {
            super::bitmap::io::consume_input(event)
                .map(|ed_cmds|
                     ed_cmds.into_iter().map(|ed_cmd|
                                             Command::Bitmap(super::bitmap::Command::Edit(ed_cmd))).collect())
        },
        &mut Editor::Chain(ref mut ed) => {
            unimplemented!()
        }
        &mut Editor::Grid(ref mut ed) => {
            unimplemented!()
        }
    }
}

pub fn eval_command(state: &mut State, command:&Command) -> Result<(), String> {
    debug!("{:?}", command);
    match (command, &mut state.editor) {
        (&Command::Bitmap(ref bc), &mut Editor::Bitmap(ref mut be)) => {
            super::bitmap::semantics::editor_eval(be, bc)
        },
        (&Command::Bitmap(ref _bc), _) => { Err("bitmap editor expected bitmap command".to_string()) }
        (_, &mut Editor::Bitmap(ref mut _be)) => { Err("bitmap command for non-bitmap editor".to_string()) }

        (&Command::Chain(ref _ch), _) => {
            unimplemented!()
        }
        (&Command::Grid(ref _gr), _) => {
            unimplemented!()
        }
    }

}


// to do: take "genesis" arguments:
// - user's self-symbol abbreviation
// - current date, time, place, etc
// - OS filesystem paths for archiving
pub fn init_state() -> State {
    let mut state_init = State {
        editor: super::types::Editor::Bitmap(
            Box::new(
                super::bitmap::Editor {
                    state: None,
                    history: vec![],
                }
            )
        ),
    };
    let init_command = Command::Bitmap(
        super::bitmap::Command::Init(
            super::bitmap::InitCommand::Make16x16
        )
    );
    eval_command(&mut state_init, &init_command);
    state_init
}

use sdl2::render::{Canvas, RenderTarget};
pub fn render_elms<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    state: &State) -> Result<render::Elms, String>
{
    match &state.editor {
        &Editor::Bitmap(ref ed) => {
            match ed.state {
                None => Ok(vec![]),
                Some(ref ed) =>
                    super::bitmap::io::render_elms(canvas, ed)
            }
        },
        &Editor::Chain(ref _ch) => {
            unimplemented!()
        }
        &Editor::Grid(ref _gr) => {
            unimplemented!()
        }
    }

}

// ideally, this "string" should have a type refinement giving the
// path abstractly as a name set, a la Fungi-Lang.
pub fn get_persis_state_path() -> String {
    let dir : String =
        std::env::current_dir().unwrap().to_str().unwrap().into();
    format!("{}/zqm.json", dir)
}

use std::fs::{File, OpenOptions};
use std::io::{BufReader, ErrorKind, Write};

pub fn load_state() -> State {
    let file = match File::open(&get_persis_state_path()) {
        Ok(f) => f,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => return init_state(),
            _ => unreachable!(),
        },
    };
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}

pub fn save_state(state: &State) -> () {
    let path = get_persis_state_path();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap();
    let output: String = serde_json::to_string_pretty(&state).unwrap();
    file.write_all(output.as_bytes()).unwrap();
}
