extern crate serde_idl;

use candid;
use menu;
use menu::MenuType;

use serde_idl::grammar::IDLProgParser;
use serde_idl::lexer::Lexer;
use serde_idl::types::{to_pretty, IDLProg, IDLType, Label};

use eval;
use types::lang::{Atom, Command, Editor, Frame, FrameCont, Name, State};

pub fn init_state() -> State {
    let (mut state_init, init_command) = {
        if true {
            let idl_spec = r#"
service server : {
  f : (a: nat, b:nat) -> () oneway;
  g : (a: text, b:nat) -> () oneway;
  h : (a: nat, b:record { nat; nat; record { nat; 0x2a:nat; nat8; }; 42:nat; 40:nat; variant{ A; 0x2a; B; C }; }) -> () oneway;
}
    "#;
            let idl_ast = candid::parse_idl(&idl_spec);
            let menu_type = candid::menutype_of_idlprog_service(&idl_ast);
            (
                State {
                    stack: vec![],
                    frame: Frame::from_editor(Editor::Menu(Box::new(menu::Editor {
                        state: None,
                        history: vec![],
                    }))),
                },
                Command::Menu(menu::Command::Init(menu::InitCommand::Default(
                    menu::MenuTree::Blank(menu_type.clone()),
                    menu_type,
                ))),
            )
        } else if false {
            use crate::bitmap;
            (
                State {
                    stack: vec![],
                    frame: Frame {
                        name: Name::Void,
                        editor: Editor::Bitmap(Box::new(bitmap::Editor {
                            state: None,
                            history: vec![],
                        })),
                        cont: FrameCont {
                            var: Name::Void,
                            commands: vec![],
                        },
                    },
                },
                Command::Bitmap(bitmap::Command::Init(bitmap::InitCommand::Make16x16)),
            )
        } else {
            use crate::menu::{MenuType, PrimType};

            let uid_person = MenuType::Product(vec![
                (
                    Name::Atom(Atom::String("uid".to_string())),
                    MenuType::Prim(PrimType::Nat),
                ),
                (
                    Name::Atom(Atom::String("first".to_string())),
                    MenuType::Prim(PrimType::Text),
                ),
                (
                    Name::Atom(Atom::String("last".to_string())),
                    MenuType::Prim(PrimType::Text),
                ),
            ]);

            let person = MenuType::Product(vec![
                (
                    Name::Atom(Atom::String("first".to_string())),
                    MenuType::Prim(PrimType::Text),
                ),
                (
                    Name::Atom(Atom::String("last".to_string())),
                    MenuType::Prim(PrimType::Text),
                ),
            ]);

            let variant_crud = MenuType::Variant(vec![
                (Name::Atom(Atom::String("create".to_string())), person),
                (
                    Name::Atom(Atom::String("remove".to_string())),
                    MenuType::Prim(PrimType::Nat),
                ),
                (Name::Atom(Atom::String("update".to_string())), uid_person),
                (
                    Name::Atom(Atom::String("delete".to_string())),
                    MenuType::Prim(PrimType::Nat),
                ),
            ]);

            let menu = MenuType::Variant(vec![
                (
                    Name::Atom(Atom::String("crud".to_string())),
                    variant_crud.clone(),
                ),
                (
                    Name::Atom(Atom::String("quit".to_string())),
                    MenuType::Prim(PrimType::Unit),
                ),
            ]);

            let config = MenuType::Product(vec![(
                Name::Atom(Atom::String("text-zoom".to_string())),
                MenuType::Prim(PrimType::Nat),
            )]);

            let root = MenuType::Variant(vec![
                (Name::Atom(Atom::String("menu".to_string())), menu),
                (Name::Atom(Atom::String("config".to_string())), config),
            ]);
            (
                State {
                    stack: vec![],
                    frame: Frame::from_editor(Editor::Menu(Box::new(menu::Editor {
                        state: None,
                        history: vec![],
                    }))),
                },
                Command::Menu(menu::Command::Init(menu::InitCommand::Default(
                    menu::MenuTree::Blank(root.clone()),
                    root,
                ))),
            )
        }
    };
    let r = eval::command_eval(&mut state_init, &init_command);
    match r {
        Ok(()) => {}
        Err(err) => eprintln!("Failed to initialize the editor: {:?}", err),
    };
    state_init
}
