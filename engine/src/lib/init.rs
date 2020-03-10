use eval;
use menu;
use types::lang::{Atom, Command, Editor, Name, State};

pub fn init_state() -> State {
    let (mut state_init, init_command) = {
        if false {
            use crate::bitmap;
            (
                State {
                    editor: Editor::Bitmap(Box::new(bitmap::Editor {
                        state: None,
                        history: vec![],
                    })),
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
                    editor: Editor::Menu(Box::new(menu::Editor {
                        state: None,
                        history: vec![],
                    })),
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
