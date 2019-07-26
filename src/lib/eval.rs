use super::{
    types::{Name,  NameFn,
            Point, Space,
            Locus, Command,
            State,
            util::{
                name_of_str, 
                namefn_id
            }
    }
};

// to do: take "genesis" arguments:
// - user's self-symbol abbreviation
// - current date, time, place, etc
// - OS filesystem paths for archiving
pub fn init_state() -> State {
    let here : Name   = name_of_str("here");
    let now  : Name   = name_of_str("now");
    let id   : NameFn = namefn_id();

    let locus_init = Locus{
        point: Point{
            place: here,
            time:  now,
        },
        space: Space {
            place: id.clone(),
            time:  id,
        },
    };
    let state_init = State {
        locus: locus_init,
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
        &Command::BeAtTime(_) => unimplemented!(),
        &Command::BeginTime(_) => unimplemented!(),
        &Command::EndTime      => unimplemented!(),
        &Command::MakePlace(_) => unimplemented!(),
        &Command::GoToPlace(_) => unimplemented!(),
        &Command::BeginPlace(_) => unimplemented!(),
        &Command::EndPlace     => unimplemented!(),
        &Command::Save(_)      => unimplemented!(),
        &Command::Restore(_)   => unimplemented!(),
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
