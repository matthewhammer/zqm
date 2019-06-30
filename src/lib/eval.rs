// Serde: Persistent state between invocations of ZQM
use serde::{Deserialize, Serialize};
use super::{
    //Name,
    types::{NameFn,
            Point,
            Space,
            Locus,
            Command}
    //Dir2D
};

#[derive(Debug, Serialize, Deserialize)]
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

pub fn eval(state: &mut State, command:&Command) -> Result<(), String> {
    match command {
        &Command::CliCommand(ref _cc) => Err("invalid command".to_string()),
        &Command::MakeTime(_) => unimplemented!(),
        &Command::MakePlace(_) => unimplemented!(),
        &Command::GotoPlace(_) => unimplemented!(),
        &Command::Save         => unimplemented!(),
        &Command::Restore      => unimplemented!(),
        &Command::Undo         => unimplemented!(),
        &Command::Redo         => unimplemented!(),
        &Command::Bitmap(ref bc) => {
            super::bitmap::semantics::editor_eval(&mut state.bitmap_editor, bc)
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
            ErrorKind::NotFound => return init(),
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
