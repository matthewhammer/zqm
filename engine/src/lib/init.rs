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

            let variant_l_r = MenuType::Variant(vec![
                (
                    Name::Atom(Atom::String("nat".to_string())),
                    MenuType::Prim(PrimType::Nat),
                ),
                (
                    Name::Atom(Atom::String("text".to_string())),
                    MenuType::Prim(PrimType::Text),
                ),
                (
                    Name::Atom(Atom::String("bool".to_string())),
                    MenuType::Prim(PrimType::Bool),
                ),
            ]);

            let product_a_b = MenuType::Product(vec![
                (
                    Name::Atom(Atom::String("apple".to_string())),
                    variant_l_r.clone(),
                ),
                (
                    Name::Atom(Atom::String("banana".to_string())),
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
                    menu::MenuTree::Blank(product_a_b.clone()),
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
