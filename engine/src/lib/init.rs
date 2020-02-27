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
            use crate::menu::{MenuChoice, MenuType, PrimType};

            let variant_l_r = MenuType::Variant(vec![
                (
                    Name::Atom(Atom::String("ll".to_string())),
                    MenuType::Prim(PrimType::Nat),
                ),
                (
                    Name::Atom(Atom::String("rr".to_string())),
                    MenuType::Prim(PrimType::Nat),
                ),
            ]);

            let product_a_b = MenuType::Product(vec![
                (
                    Name::Atom(Atom::String("aa".to_string())),
                    variant_l_r.clone(),
                ),
                (
                    Name::Atom(Atom::String("bb".to_string())),
                    variant_l_r.clone(),
                ),
            ]);

            (
                State {
                    editor: Editor::Menu(Box::new(menu::Editor {
                        state: None,
                        history: vec![],
                    })),
                },
                Command::Menu(menu::Command::Init(menu::InitCommand::Default(
                    MenuChoice::Blank,
                    product_a_b,
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
