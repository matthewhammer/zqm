use serde::{Deserialize, Serialize};
use std::rc::Rc;
use types::lang::Name;

pub type Text = String;
pub type Nat = usize; // todo -- use a bignum rep
pub type Label = Name;

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum MenuType {
    Prim(PrimType),
    Variant(Vec<(Label, MenuType)>),
    Product(Vec<(Label, MenuType)>),
    Option(Box<MenuType>),
    Vec(Box<MenuType>),
    Tup(Vec<MenuType>),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum PrimType {
    Nat,
    Text,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum MenuChoice {
    Blank,
    Nat(Nat),
    Text(Text),
    Variant(Label, Box<MenuChoice>),
    Product(Vec<(Label, MenuChoice)>),
    Some(Box<MenuChoice>),
    None,
    Vec(Vec<MenuChoice>),
    Tup(Vec<MenuChoice>),
    Error(Error, Box<MenuChoice>),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum Error {
    MenuTypeMismatch(MenuType, Tag), // found vs expected
    Blank(Tag),                      // found blank vs expected completed 'tag'
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum Tag {
    Prim(PrimType),
    Variant,
    Product,
    Option,
    Vec,
    Tup,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum InitCommand {
    //DefaultChoice(MenuChoice, MenuType)
    Default(MenuChoice, MenuType),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum AutoCommand {
    CheckMenuType,
    CheckComplete,
    Replace(MenuChoice),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum EditCommand {
    GotoRoot,           // Escape
    AutoFill,           // Tab
    NextBlank,          // ArrowDown
    PrevBlank,          // ArrowUp
    VecInsertBlank,     // Comma
    VecInsertAuto,      // Shift-Comma
    VariantNext,        // ArrowDown
    VariantPrev,        // ArrowUp
    VariantAccept,      // Enter
    VariantReset,       // Space
    Choose(MenuChoice), // (no simple keybinding)
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum Command {
    Init(InitCommand),
    Auto(AutoCommand),
    Edit(EditCommand),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum MenuCtx {
    Root,
    Product(LabelChoice<Box<MenuCtx>>),
    Variant(LabelChoice<Box<MenuCtx>>),
    Option(bool, Box<MenuCtx>),
    Vec(PosChoice<Box<MenuCtx>>),
    Tup(PosChoice<Box<MenuCtx>>),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct PosChoice<X> {
    before_choice: Vec<(MenuTree, MenuType)>,
    choice: Option<X>,
    after_choice: Vec<(MenuTree, MenuType)>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct LabelChoice<X> {
    before_choice: Vec<(Label, MenuTree, MenuType)>,
    choice: Option<(Label, X)>,
    after_choice: Vec<(Label, MenuTree, MenuType)>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum MenuTree {
    Product(Vec<(Label, MenuTree, MenuType)>),
    Variant(LabelChoice<Box<(MenuTree, MenuType)>>),
    Option(bool, Box<MenuTree>, MenuType),
    Vec(Vec<MenuTree>, MenuType),
    Tup(Vec<(MenuTree, MenuType)>),
    Blank(MenuType),
    Nat(Nat),
    Text(Text),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct MenuState {
    pub typ: Rc<MenuType>, // invariant: typ = unfocus(ctx, tree_typ);
    pub ctx: MenuCtx,      // invariant: see typ.
    pub tree: MenuTree,    // invariant: tree has type tree_typ.
    pub tree_typ: MenuType,
    pub choice: MenuChoice, // invariant: choice has type typ; choice = unfocus(ctx, tree);
                            // invariant: tree has type typ
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Editor {
    pub state: Option<MenuState>,
    pub history: Vec<Command>,
}

pub mod semantics {
    use super::*;

    pub type Err = String;
    pub type Res = Result<(), Err>;

    pub fn editor_eval(menu: &mut Editor, command: &Command) -> Res {
        info!("editor_eval({:?}) begin", command);
        let res = match command {
            Command::Init(InitCommand::Default(ref default_choice, ref typ)) => {
                menu.state = Some(MenuState {
                    typ: Rc::new(typ.clone()),
                    ctx: MenuCtx::Root,
                    tree: MenuTree::Blank(typ.clone()),
                    tree_typ: typ.clone(),
                    choice: default_choice.clone(),
                });
                Ok(())
            }
            Command::Edit(ref c) => match menu.state {
                None => Err("Invalid editor state".to_string()),
                Some(ref mut st) => state_eval_command(st, c),
            },
            Command::Auto(ref c) => unimplemented!(),
        };
        info!("editor_eval({:?}) ==> {:?}", command, res);
        menu.history.push(command.clone());
        res
    }

    pub fn state_eval_command(menu: &mut MenuState, command: &EditCommand) -> Res {
        match command {
            &EditCommand::GotoRoot => goto_root(menu),
            &EditCommand::AutoFill => {
                let tree = auto_fill(&menu.typ, 1);
                menu.tree = tree_union(&menu.tree, &tree);
                Ok(())
            }
            &EditCommand::NextBlank => next_blank(menu).map(|_| ()),
            &EditCommand::PrevBlank => unimplemented!(),
            &EditCommand::VecInsertBlank => {
                assert_tree_tag(&menu.tree, &Tag::Vec)?;
                unimplemented!()
            }
            &EditCommand::VecInsertAuto => {
                assert_tree_tag(&menu.tree, &Tag::Vec)?;
                state_eval_command(menu, &EditCommand::VecInsertBlank)?;
                state_eval_command(menu, &EditCommand::AutoFill)
            }
            &EditCommand::VariantNext => {
                assert_tree_tag(&menu.tree, &Tag::Variant)?;
                next_subtree_choice(menu)
            }
            &EditCommand::VariantAccept => {
                assert_tree_tag(&menu.tree, &Tag::Variant)?;
                ascend(menu)
            }
            _ => unimplemented!(),
        }
    }

    pub fn assert_tree_tag(tree: &MenuTree, tag: &Tag) -> Res {
        unimplemented!()
    }

    pub fn tree_has_complete_choice(tree: &MenuTree) -> bool {
        match tree {
            MenuTree::Product(trees) => unimplemented!(),
            MenuTree::Variant(choice) => match &choice.choice {
                None => false,
                Some((l, t_tt)) => tree_has_complete_choice(&t_tt.0),
            },
            _ => unimplemented!(),
        }
    }

    pub fn ascend(menu: &mut MenuState) -> Res {
        unimplemented!()
    }

    pub fn goto_root(menu: &mut MenuState) -> Res {
        match menu.ctx {
            MenuCtx::Root => Ok(()),
            _ => {
                ascend(menu)?;
                goto_root(menu)
            }
        }
    }

    pub fn next_blank(menu: &mut MenuState) -> Result<MenuType, Err> {
        match menu.tree {
            MenuTree::Blank(ref t) => Ok(t.clone()),
            _ => {
                next_tree(menu)?;
                next_blank(menu)
            }
        }
    }

    pub fn next_tree(menu: &mut MenuState) -> Res {
        match next_subtree(menu) {
            Ok(()) => Ok(()),
            // Err case: Need to look for next tree in the context:
            Err(_) => match next_sibling(menu) {
                Ok(()) => Ok(()),
                Err(_) => {
                    ascend(menu)?;
                    next_tree(menu)
                }
            },
        }
    }

    pub fn next_subtree(menu: &mut MenuState) -> Res {
        // navigate product field structure; ignore unchosen variant options.
        match menu.tree {
            MenuTree::Product(ref trees) => {
                let mut trees = trees.clone();
                if trees.len() > 0 {
                    trees.rotate_left(1);
                    let (label, tree, tree_t) = trees.pop().unwrap();
                    menu.tree = tree;
                    menu.tree_typ = tree_t;
                    menu.ctx = MenuCtx::Product(LabelChoice {
                        before_choice: vec![],
                        choice: Some((label, Box::new(menu.ctx.clone()))),
                        after_choice: trees,
                    });
                    Ok(())
                } else {
                    Err("no subtrees".to_string())
                }
            }
            _ => unimplemented!(),
        }
    }

    // consult the context, and refocus the ctx/tree/typ fields
    // on the next product/vec/tuple arm,
    // or Err if no such list in immediate context.
    pub fn next_sibling(menu: &mut MenuState) -> Res {
        match menu.ctx {
            MenuCtx::Product(ref ctx) => {
                if ctx.after_choice.len() > 0 {
                    let mut ctx = ctx.clone();
                    ctx.after_choice.rotate_left(1);
                    let (label, tree, tree_typ) = ctx.after_choice.pop().unwrap();
                    unimplemented!()
                } else {
                    Err("no siblings".to_string())
                }
            }
            _ => unimplemented!(),
        }
    }

    pub fn next_subtree_choice(menu: &mut MenuState) -> Res {
        // select successive variant choices; ignore product structure.
        unimplemented!()
    }

    pub fn next_sibling_choice(menu: &mut MenuState) -> Res {
        // select successive variant choices from context; ignore product structure.
        unimplemented!()
    }

    pub fn next_tree_choice(menu: &mut MenuState) -> Res {
        match next_subtree_choice(menu) {
            Ok(()) => Ok(()),
            // Err case: Need to look for next tree in the context:
            Err(_) => match next_sibling_choice(menu) {
                Ok(()) => Ok(()),
                Err(_) => {
                    ascend(menu)?;
                    next_tree_choice(menu)
                }
            },
        }
    }

    pub fn tree_union(tree1: &MenuTree, tree2: &MenuTree) -> MenuTree {
        // todo -- assert that the blanks' types agree
        match (tree1, tree2) {
            (&MenuTree::Blank(_), _) => tree2.clone(),
            (_, &MenuTree::Blank(_)) => tree2.clone(),
            (_, _) => {
                info!("tree_union({:?}, {:?}) begin", tree1, tree2);
                unimplemented!()
            }
        }
    }

    pub fn auto_fill(typ: &MenuType, depth: usize) -> MenuTree {
        if depth == 0 {
            MenuTree::Blank(typ.clone())
        } else {
            match typ {
                &MenuType::Prim(PrimType::Nat) => MenuTree::Nat(0),
                &MenuType::Prim(PrimType::Text) => MenuTree::Text("".to_string()),
                &MenuType::Variant(ref labtyps) => {
                    let after_choices: Vec<(Label, MenuTree, MenuType)> = labtyps
                        .iter()
                        .map(|(l, lt)| (l.clone(), auto_fill(lt, depth - 1), lt.clone()))
                        .collect();
                    MenuTree::Variant(LabelChoice {
                        before_choice: vec![],
                        choice: None,
                        after_choice: after_choices,
                    })
                }
                &MenuType::Product(ref labtyps) => {
                    let menus: Vec<(Label, MenuTree, MenuType)> = labtyps
                        .iter()
                        .map(|(l, lt)| (l.clone(), auto_fill(lt, depth - 1), lt.clone()))
                        .collect();
                    MenuTree::Product(menus)
                }
                &MenuType::Vec(ref typ) => {
                    if depth == 1 {
                        MenuTree::Vec(vec![], *typ.clone())
                    } else {
                        MenuTree::Vec(vec![auto_fill(typ, depth - 1)], *typ.clone())
                    }
                }
                &MenuType::Option(ref typ) => {
                    let tree = auto_fill(typ, depth - 1);
                    MenuTree::Option(false, Box::new(tree), *typ.clone())
                }
                &MenuType::Tup(ref typs) => {
                    let menus: Vec<(MenuTree, MenuType)> = typs
                        .iter()
                        .map(|t| (auto_fill(t, depth - 1), t.clone()))
                        .collect();
                    MenuTree::Tup(menus)
                }
            }
        }
    }

    pub fn get_choice(menu: &MenuState) -> Result<MenuChoice, Error> {
        #[allow(unused_variables, unused_mut)]
        pub fn choice_of_ctx(ctx: &MenuCtx, local_choice: MenuChoice) -> Result<MenuChoice, Error> {
            unimplemented!()
        };

        #[allow(unused_variables, unused_mut)]
        pub fn choice_of_tree(tree: &MenuTree) -> Result<MenuChoice, Error> {
            unimplemented!()
        };

        let choice = choice_of_tree(&menu.tree)?;
        let choice = choice_of_ctx(&menu.ctx, choice)?;
        Ok(choice)
    }
}

pub mod io {
    use super::{EditCommand, Label, MenuCtx, MenuState, MenuTree};
    use render::Render;
    use types::event::Event;
    use types::{
        lang::{Dir2D, Name},
        render::{Color, Dim, Elms, Fill},
    };

    pub fn edit_commands_of_event(event: &Event) -> Result<Vec<EditCommand>, ()> {
        match event {
            &Event::Quit { .. } => Err(()),
            &Event::KeyDown(ref kei) => match kei.key.as_str() {
                "Escape" => Err(()),
                " " => Ok(vec![]),
                "Tab" => Ok(vec![EditCommand::AutoFill]),
                "ArrowLeft" => Ok(vec![EditCommand::PrevBlank]),
                "ArrowRight" => Ok(vec![EditCommand::NextBlank]),
                "ArrowUp" => Ok(vec![EditCommand::PrevBlank]),
                "ArrowDown" => Ok(vec![EditCommand::NextBlank]),
                _ => Ok(vec![]),
            },
            _ => Ok(vec![]),
        }
    }

    pub fn render_elms(menu: &MenuState) -> Result<Elms, String> {
        use crate::render::{FlowAtts, FrameType, TextAtts};

        // eventually we get these atts from
        //  some environment-determined settings
        fn text_atts() -> TextAtts {
            TextAtts {
                zoom: 3,
                fg_fill: Fill::Closed(Color::RGB(255, 255, 255)),
                bg_fill: Fill::Closed(Color::RGB(30, 0, 0)),
                glyph_dim: Dim {
                    width: 5,
                    height: 5,
                },
                glyph_flow: FlowAtts {
                    dir: Dir2D::Right,
                    padding: 3,
                },
            }
        };
        fn blank_atts() -> TextAtts {
            TextAtts {
                zoom: 3,
                fg_fill: Fill::Closed(Color::RGB(255, 200, 200)),
                bg_fill: Fill::Closed(Color::RGB(100, 0, 0)),
                glyph_dim: Dim {
                    width: 5,
                    height: 5,
                },
                glyph_flow: FlowAtts {
                    dir: Dir2D::Right,
                    padding: 3,
                },
            }
        };
        fn ctx_flow() -> FlowAtts {
            FlowAtts {
                dir: Dir2D::Left,
                padding: 2,
            }
        };
        fn tree_flow() -> FlowAtts {
            FlowAtts {
                dir: Dir2D::Right,
                padding: 2,
            }
        };
        fn sub_flow() -> FlowAtts {
            FlowAtts {
                dir: Dir2D::Down,
                padding: 1,
            }
        };
        // eventually we want smarter layout algorithms
        fn tup_flow() -> FlowAtts {
            sub_flow()
        };
        fn vec_flow() -> FlowAtts {
            sub_flow()
        };
        fn menu_flow() -> FlowAtts {
            tree_flow()
        };

        fn render_product_label(label: &Label, r: &mut Render) {
            r.name(label, &text_atts());
            r.str("=", &text_atts());
        }

        fn render_variant_label(label: &Label, r: &mut Render) {
            r.str("#", &text_atts());
            r.name(label, &text_atts());
            r.str("=", &text_atts());
        }

        fn begin_item(r: &mut Render) {
            let tree_flow = FlowAtts {
                dir: Dir2D::Right,
                padding: 2,
            };
            r.begin(&Name::Void, FrameType::Flow(tree_flow))
        }

        fn render_ctx(ctx: &MenuCtx, r: &mut Render) {
            info!("render_ctx({:?})", ctx);
            let mut next_ctx = None;
            r.begin(&Name::Void, FrameType::Flow(tree_flow()));
            match ctx {
                &MenuCtx::Root => {
                    r.str("/", &text_atts());
                }
                &MenuCtx::Product(ref ch) => {
                    r.begin(&Name::Void, FrameType::Flow(sub_flow()));
                    for (l, t, ty) in ch.before_choice.iter() {
                        begin_item(r);
                        render_product_label(l, r);
                        render_tree(t, r);
                        r.end();
                    }
                    if let Some((ref l, ref ctx)) = ch.choice {
                        begin_item(r);
                        render_product_label(l, r);
                        r.str(" ...", &text_atts());
                        r.end();
                        next_ctx = Some((*ctx).clone());
                    };
                    for (l, t, ty) in ch.after_choice.iter() {
                        begin_item(r);
                        render_product_label(&l, r);
                        render_tree(t, r);
                        r.end();
                    }
                    r.end();
                }
                &MenuCtx::Variant(ref ch) => {
                    r.begin(&Name::Void, FrameType::Flow(sub_flow()));
                    for (l, t, ty) in ch.before_choice.iter() {
                        begin_item(r);
                        render_variant_label(&l, r);
                        render_tree(t, r);
                        r.end()
                    }
                    if let Some((ref l, ref ctx)) = ch.choice {
                        begin_item(r);
                        render_variant_label(l, r);
                        r.str(" ...", &text_atts());
                        render_ctx(&*ctx, r);
                        r.end();
                        next_ctx = Some((*ctx).clone());
                    };
                    for (l, t, ty) in ch.after_choice.iter() {
                        begin_item(r);
                        render_variant_label(&l, r);
                        render_tree(t, r);
                        r.end()
                    }
                    r.end();
                }
                &MenuCtx::Option(flag, ref body) => unimplemented!(),
                &MenuCtx::Vec(ref ch) => unimplemented!(),
                &MenuCtx::Tup(ref ch) => unimplemented!(),
            };
            r.end();
            // continue rendering the rest of the context, in whatever flow we are using for that purpose.
            if let Some(ctx) = next_ctx {
                info!("context continues...");
                render_ctx(&ctx, r)
            } else {
                info!("context end: root.");
            };
        };

        fn render_tree(tree: &MenuTree, r: &mut Render) {
            //info!("render_tree({:?}): begin", tree);
            r.begin(&Name::Void, FrameType::Flow(tree_flow()));
            match tree {
                &MenuTree::Product(ref fields) => {
                    r.begin(&Name::Void, FrameType::Flow(sub_flow()));
                    for (l, t, ty) in fields.iter() {
                        begin_item(r);
                        render_product_label(l, r);
                        render_tree(t, r);
                        r.end()
                    }
                    r.end()
                }
                &MenuTree::Variant(ref ch) => {
                    r.begin(&Name::Void, FrameType::Flow(sub_flow()));
                    for (l, t, ty) in ch.before_choice.iter() {
                        begin_item(r);
                        render_variant_label(l, r);
                        render_tree(t, r);
                        r.end()
                    }
                    if let Some((ref l, ref tree)) = ch.choice {
                        begin_item(r);
                        render_variant_label(l, r);
                        render_tree(&tree.0, r);
                        r.end();
                    };
                    for (l, t, ty) in ch.after_choice.iter() {
                        begin_item(r);
                        render_variant_label(l, r);
                        render_tree(t, r);
                        r.end()
                    }
                    r.end()
                }
                &MenuTree::Option(flag, ref tree, ref typ) => {
                    if flag {
                        r.str("?", &text_atts())
                    };
                    render_tree(&*tree, r)
                }
                &MenuTree::Vec(ref trees, ref _typ) => {
                    r.begin(&Name::Void, FrameType::Flow(vec_flow()));
                    for tree in trees.iter() {
                        render_tree(tree, r)
                    }
                    r.end();
                }
                &MenuTree::Tup(ref trees) => {
                    r.begin(&Name::Void, FrameType::Flow(tup_flow()));
                    for (tree, _typ) in trees.iter() {
                        render_tree(tree, r)
                    }
                    r.end();
                }
                &MenuTree::Blank(ref typ) => r.str("__<blank>__", &blank_atts()),
                &MenuTree::Nat(n) => r.text(&format!("{}", n), &text_atts()),
                &MenuTree::Text(ref t) => r.text(t, &text_atts()),
            };
            //info!("render_tree({:?}): end.", tree);
            r.end();
        };
        info!("render_menu_begin");
        let mut r = Render::new();
        r.begin(&Name::Void, FrameType::Flow(menu_flow()));

        r.begin(&Name::Void, FrameType::Flow(ctx_flow()));
        render_ctx(&menu.ctx, &mut r);
        r.end();

        r.begin(&Name::Void, FrameType::Flow(tree_flow()));
        render_tree(&menu.tree, &mut r);
        r.end();

        r.end();
        info!("render_menu_end");
        Ok(r.into_elms())
    }
}
