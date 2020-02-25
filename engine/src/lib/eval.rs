// to-do/question: rename this module to 'engine'?

use bitmap;
use menu;

pub use super::types::{
    event::Event,
    lang::{Command, Editor, State},
    render,
};

pub fn commands_of_event(state: &mut State, event: &Event) -> Result<Vec<Command>, ()> {
    debug!("commands_of_event {:?}", event);
    let res = match &mut state.editor {
        &mut Editor::Bitmap(ref _ed) => {
            // to do -- insert a name into each command that is unique,
            // but whose structure encodes a wallclock timestamp, among other sequence numbers.
            bitmap::io::edit_commands_of_event(event).map(|ed_cmds| {
                ed_cmds
                    .into_iter()
                    .map(|ed_cmd| Command::Bitmap(bitmap::Command::Edit(ed_cmd)))
                    .collect()
            })
        }
        &mut Editor::Menu(ref _ed) => {
            // to do -- insert a name into each command that is unique,
            // but whose structure encodes a wallclock timestamp, among other sequence numbers.
            menu::io::edit_commands_of_event(event).map(|ed_cmds| {
                ed_cmds
                    .into_iter()
                    .map(|ed_cmd| Command::Menu(menu::Command::Edit(ed_cmd)))
                    .collect()
            })
        }
        &mut Editor::Chain(ref mut _ed) => unimplemented!(),
        &mut Editor::Grid(ref mut _ed) => unimplemented!(),
    };
    debug!("commands_of_event {:?} ==> {:?}", event, res);
    res
}

pub fn command_eval(state: &mut State, command: &Command) -> Result<(), String> {
    debug!("command_eval {:?}", command);
    let res = match (command, &mut state.editor) {
        (&Command::Bitmap(ref bc), &mut Editor::Bitmap(ref mut be)) => {
            super::bitmap::semantics::editor_eval(be, bc)
        }
        (&Command::Bitmap(ref _bc), _) => Err("bitmap editor expected bitmap command".to_string()),
        (_, &mut Editor::Bitmap(ref mut _be)) => {
            Err("bitmap command for non-bitmap editor".to_string())
        }

        (&Command::Menu(ref c), &mut Editor::Menu(ref mut e)) => {
            super::menu::semantics::editor_eval(e, c)
        }
        (&Command::Menu(ref _c), _) => Err("menu editor expected menu command".to_string()),
        (_, &mut Editor::Menu(ref mut _e)) => Err("menu command for non-menu editor".to_string()),

        (&Command::Chain(ref _ch), _) => unimplemented!(),
        (&Command::Grid(ref _gr), _) => unimplemented!(),
    };
    debug!("command_eval {:?} ==> {:?}", command, res);
    res
}

pub fn init_state() -> State {
    let (mut state_init, init_command) = {
        if false {
            (
                State {
                    editor: super::types::lang::Editor::Bitmap(Box::new(crate::bitmap::Editor {
                        state: None,
                        history: vec![],
                    })),
                },
                Command::Bitmap(crate::bitmap::Command::Init(
                    crate::bitmap::InitCommand::Make16x16,
                )),
            )
        } else {
            use crate::menu::{self, MenuChoice, MenuType, PrimType};
            use crate::types::lang::{Atom, Name};

            let typ = MenuType::Product(vec![(
                Name::Atom(Atom::String("a".to_string())),
                MenuType::Variant(vec![
                    (
                        Name::Atom(Atom::String("l".to_string())),
                        MenuType::Prim(PrimType::Nat),
                    ),
                    (
                        Name::Atom(Atom::String("r".to_string())),
                        MenuType::Prim(PrimType::Nat),
                    ),
                ]),
            )]);

            (
                State {
                    editor: super::types::lang::Editor::Menu(Box::new(menu::Editor {
                        state: None,
                        history: vec![],
                    })),
                },
                Command::Menu(menu::Command::Init(menu::InitCommand::Default(
                    MenuChoice::Blank,
                    typ,
                ))),
            )
        }
    };
    let r = command_eval(&mut state_init, &init_command);
    match r {
        Ok(()) => {}
        Err(err) => eprintln!("Failed to initialize the editor: {:?}", err),
    };
    state_init
}

pub fn render_elms(state: &State) -> Result<render::Elms, String> {
    match &state.editor {
        &Editor::Bitmap(ref ed) => match ed.state {
            None => Ok(vec![]),
            Some(ref ed) => super::bitmap::io::render_elms(ed),
        },
        &Editor::Menu(ref ed) => match ed.state {
            None => Ok(vec![]),
            Some(ref st) => super::menu::io::render_elms(st),
        },
        &Editor::Chain(ref _ch) => unimplemented!(),
        &Editor::Grid(ref _gr) => unimplemented!(),
    }
}

pub fn get_persis_state_path() -> String {
    let dir: String = std::env::current_dir().unwrap().to_str().unwrap().into();
    format!("{}/zqm.json", dir)
}

use std::fs::{File, OpenOptions};
use std::io::{BufReader, ErrorKind, Write};

pub fn load_state() -> State {
    let path = &get_persis_state_path();
    let file = match File::open(path) {
        Ok(f) => f,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => return init_state(),
            _ => unreachable!(),
        },
    };
    info!("Loading from {:?}", path);
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
