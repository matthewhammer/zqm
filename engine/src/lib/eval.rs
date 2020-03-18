// to-do/question: rename this module to 'engine'?

use bitmap;
use candid;
use menu;

pub enum LoadState {
    CandidFile { file: String },
    Resume,
}

pub use super::types::{
    event::Event,
    lang::{Command, Editor, Media, State},
    render,
};

pub fn commands_of_event(state: &mut State, event: &Event) -> Result<Vec<Command>, ()> {
    debug!("commands_of_event {:?}", event);
    let res = match &mut state.frame.editor {
        &mut Editor::CandidRepl(ref _repl) => unimplemented!(),
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
        &mut Editor::Menu(ref mut ed) => {
            // to do -- insert a name into each command that is unique,
            // but whose structure encodes a wallclock timestamp, among other sequence numbers.
            match ed.state {
                Some(ref mut st) =>
  //              Editor::Menu(ref mut menu_state) => {
                    menu::io::edit_commands_of_event(st, event).map(|ed_cmds| {
                        ed_cmds
                            .into_iter()
                            .map(|ed_cmd| Command::Menu(menu::Command::Edit(ed_cmd)))
                            .collect()
                    }),

                    _ => unreachable!()
            }
        }
        &mut Editor::Chain(ref mut _ed) => unimplemented!(),
        &mut Editor::Grid(ref mut _ed) => unimplemented!(),
    };
    debug!("commands_of_event {:?} ==> {:?}", event, res);
    res
}

use serde_idl::value::{IDLArgs, IDLField, IDLValue};

pub fn command_eval(state: &mut State, command: &Command) -> Result<(), String> {
    if let &Command::Return(ref media) = command {
        debug!("command_eval: return...");

        if state.stack.len() == 0 {
            Err(format!("cannot return; empty stack"))
        } else {
            let old_top = state.frame.clone();
            state.frame = state.stack.pop().unwrap();
            let resume = Command::Resume(media.clone(), old_top);
            command_eval(state, &resume)
        }
    } else if let &Command::Resume(Media::MenuTree(ref mt), ref old_frame) = command {
        debug!("command_eval: resume, with menu tree...");

        // to do: clean, refactor and move this logic (e.g., into the candid module)
        match &**mt {
            &menu::MenuTree::Variant(ref ch) => match &ch.choice {
                None => {
                    warn!("no choice selected; try again...");
                }
                &Some((ref lab, ref tree, ref typ)) => {
                    let method = format!("{}", lab);
                    let str = format!("({})", tree);
                    info!("Sending message \"{}\", with args {}", method, str);
                    let args = &str.parse::<IDLArgs>().unwrap();
                    debug!("...as {}({})", method, args);

                    if let &mut Editor::CandidRepl(ref mut repl) = &mut state.frame.editor {
                        use ic_http_agent::{Blob, CanisterId, Waiter};
                        use std::time::Duration;
                        use tokio::runtime::Runtime;

                        info!(
                            "...to canister_id {:?} at replica_url {:?}",
                            repl.config.canister_id, repl.config.replica_url
                        );

                        let mut runtime = Runtime::new().expect("Unable to create a runtime");
                        let waiter = Waiter::builder()
                            .throttle(Duration::from_millis(100))
                            .timeout(Duration::from_secs(60))
                            .build();
                        let agent = candid::agent(&repl.config.replica_url).unwrap();
                        let canister_id =
                            CanisterId::from_text(repl.config.canister_id.clone()).unwrap();
                        let timestamp = std::time::SystemTime::now();
                        let blob_res = runtime.block_on(agent.call_and_wait(
                            &canister_id,
                            &method,
                            &Blob(args.to_bytes().unwrap()),
                            waiter,
                        ));

                        let elapsed = timestamp.elapsed().unwrap();
                        if let Ok(blob_res) = blob_res {
                            let result = serde_idl::IDLArgs::from_bytes(&(*blob_res.unwrap().0));
                            let res = format!("{:?}", result.unwrap().args);
                            info!("..successful result {:?}", res);
                            let call = candid::Call {
                                timestamp: timestamp,
                                duration: elapsed,
                                method: method,
                                args: tree.clone(),
                                args_idl: str.to_string(),
                                rets_idl: Ok(res),
                                rets: Err("<to do>".to_string()),
                            };
                            repl.history.push(call)
                        } else {
                            let res = format!("{:?}", blob_res);
                            info!("..error result {:?}", res);
                            let call = candid::Call {
                                timestamp: timestamp,
                                duration: elapsed,
                                method: method,
                                args: tree.clone(),
                                args_idl: str.to_string(),
                                rets_idl: Err(res),
                                rets: Err("<to do>".to_string()),
                            };
                            repl.history.push(call)
                        }
                    } else {
                        unreachable!()
                    }
                }
            },
            _ => unreachable!("broken invariants"),
        }
        state.stack.push(state.frame.clone());
        state.frame = old_frame.clone();
        Ok(())
    } else {
        debug!("command_eval {:?}", command);
        info!("{:?}", command);
        let res = match (command, &mut state.frame.editor) {
            (&Command::Bitmap(ref bc), &mut Editor::Bitmap(ref mut be)) => {
                super::bitmap::semantics::editor_eval(be, bc)
            }
            (&Command::Bitmap(ref _bc), _) => {
                Err("bitmap editor expected bitmap command".to_string())
            }
            (_, &mut Editor::Bitmap(ref mut _be)) => {
                Err("bitmap command for non-bitmap editor".to_string())
            }

            (&Command::Menu(ref c), &mut Editor::Menu(ref mut e)) => {
                match super::menu::semantics::editor_eval(e, c) {
                    Ok(()) => Ok(()),
                    Err(menu::Halt::Commit(mt)) => {
                        command_eval(state, &Command::Return(Media::MenuTree(Box::new(mt))))
                    }
                    Err(menu::Halt::Message(m)) => Err(m),
                }
            }
            (&Command::Menu(ref _c), _) => Err("menu editor expected menu command".to_string()),
            (_, &mut Editor::Menu(ref mut _e)) => {
                Err("menu command for non-menu editor".to_string())
            }

            (&Command::Chain(ref _ch), _) => unimplemented!(),
            (&Command::Grid(ref _gr), _) => unimplemented!(),
            (&Command::Return(_), _) => unimplemented!(),
            (&Command::Resume(_, _), _) => unimplemented!(),
        };
        debug!("command_eval {:?} ==> {:?}", command, res);
        res
    }
}

use crate::render::{FlowAtts, FrameType, Render, TextAtts};
use types::lang::{Dir2D, Name};

pub fn render_elms_of_editor(editor: &Editor, r: &mut Render) {
    match editor {
        &Editor::CandidRepl(ref repl) => candid::render_elms(repl, r),
        &Editor::Bitmap(ref ed) => match ed.state {
            None => warn!("to do: render empty bitmap editor?"),
            Some(ref ed) => unimplemented!(), //super::bitmap::io::render_elms(ed, r),
        },
        &Editor::Menu(ref ed) => match ed.state {
            None => warn!("to do: render empty bitmap editor?"),
            Some(ref st) => menu::io::render_elms(st, r),
        },
        &Editor::Chain(ref _ch) => unimplemented!(),
        &Editor::Grid(ref _gr) => unimplemented!(),
    }
}

pub fn render_elms(state: &State) -> Result<render::Elms, String> {
    let mut r = Render::new();
    fn vert_flow() -> FlowAtts {
        FlowAtts {
            dir: Dir2D::Down,
            intra_pad: 2,
            inter_pad: 2,
        }
    }
    fn horz_flow() -> FlowAtts {
        FlowAtts {
            dir: Dir2D::Right,
            intra_pad: 2,
            inter_pad: 2,
        }
    }
    // to do: acquire and use the screen dimension for "clip flows".
    r.begin(&Name::Void, FrameType::Flow(horz_flow()));
    r.begin(&Name::Void, FrameType::Flow(vert_flow()));
    for frame in state.stack.iter() {
        render_elms_of_editor(&frame.editor, &mut r);
    }
    r.end();
    render_elms_of_editor(&state.frame.editor, &mut r);
    r.end();
    Ok(r.into_elms())
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
            ErrorKind::NotFound => return crate::init::init_state(),
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
