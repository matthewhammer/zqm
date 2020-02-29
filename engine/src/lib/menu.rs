use serde::{Deserialize, Serialize};
use std::rc::Rc;
use types::lang::{Dir1D, Name};

pub type Text = String;
pub type Nat = usize; // todo -- use a bignum rep
pub type Label = Name;

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum MenuType {
    Prim(PrimType),
    Variant(Vec<(Label, MenuType)>),
    Product(Vec<(Label, MenuType)>),
    Option(Box<MenuType>),
    Vec(Box<MenuType>),
    Tup(Vec<MenuType>),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum PrimType {
    Nat,
    Text,
    Bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum MenuTree {
    Product(Vec<(Label, MenuTree, MenuType)>),
    Variant(Box<LabelChoice>),
    Option(bool, Box<MenuTree>, MenuType),
    Vec(Vec<MenuTree>, MenuType),
    Tup(Vec<(MenuTree, MenuType)>),
    Blank(MenuType),
    Nat(Nat),
    Text(Text),
    Bool(bool),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct LabelSelect {
    before: Vec<(Label, MenuTree, MenuType)>,
    ctx: MenuCtx,
    label: Label,
    after: Vec<(Label, MenuTree, MenuType)>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct PosSelect {
    before: Vec<(Label, MenuTree, MenuType)>,
    ctx: MenuCtx,
    after: Vec<(Label, MenuTree, MenuType)>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct LabelChoice {
    before: Vec<(Label, MenuTree, MenuType)>,
    choice: Option<(Label, MenuTree, MenuType)>,
    after: Vec<(Label, MenuTree, MenuType)>,
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
    Default(MenuTree, MenuType),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum AutoCommand {
    CheckMenuType,
    CheckComplete,
    Replace(MenuTree),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum EditCommand {
    GotoRoot,  // Escape
    AutoFill,  // Tab
    NextTree,  // ArrowRight
    PrevTree,  // ArrowLeft
    NextBlank, // ArrowDown
    PrevBlank, // ArrowUp
    VariantNext,
    VecInsertBlank, // Comma
    VecInsertAuto,  // Shift-Comma
    VariantAccept,  // Enter
    VariantReset,   // Space
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
    Product(Box<LabelSelect>),
    Variant(Box<LabelSelect>),
    Option(bool, Box<MenuCtx>),
    Vec(Box<PosSelect>),
    Tup(Box<PosSelect>),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct MenuState {
    pub typ: Rc<MenuType>, // invariant: typ = unfocus(ctx, tree_typ);
    pub ctx: MenuCtx,      // invariant: see typ.
    pub tree: MenuTree,    // invariant: tree has type tree_typ.
    pub tree_typ: MenuType,
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
                drop(tree_union(&menu.tree, &tree));
                menu.tree = tree;
                Ok(())
            }
            &EditCommand::NextBlank => next_blank(menu).map(|_| ()),
            &EditCommand::PrevBlank => prev_blank(menu).map(|_| ()),
            &EditCommand::NextTree => next_tree(menu).map(|_| ()),
            &EditCommand::PrevTree => prev_tree(menu).map(|_| ()),
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

    pub fn tree_is_complete(tree: &MenuTree) -> bool {
        match tree {
            MenuTree::Product(trees) => unimplemented!(),
            MenuTree::Variant(choice) => match &choice.choice {
                None => false,
                Some((l, t, t_tt)) => tree_is_complete(t),
            },
            _ => unimplemented!(),
        }
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

    pub fn prev_blank(menu: &mut MenuState) -> Result<MenuType, Err> {
        match menu.tree {
            MenuTree::Blank(ref t) => Ok(t.clone()),
            _ => {
                prev_tree(menu)?;
                prev_blank(menu)
            }
        }
    }

    pub fn next_tree(menu: &mut MenuState) -> Res {
        match descend(menu, Dir1D::Forward) {
            Ok(()) => Ok(()),
            Err(_) => match next_sibling(menu) {
                Ok(()) => Ok(()),
                Err(_) => {
                    ascend(menu)?;
                    next_tree(menu)
                }
            },
        }
    }

    pub fn prev_tree(menu: &mut MenuState) -> Res {
        match descend(menu, Dir1D::Backward) {
            Ok(()) => Ok(()),
            Err(_) => match prev_sibling(menu) {
                Ok(()) => Ok(()),
                Err(_) => {
                    ascend(menu)?;
                    prev_tree(menu)
                }
            },
        }
    }

    pub fn ascend(menu: &mut MenuState) -> Res {
        unimplemented!()
    }

    pub fn descend(menu: &mut MenuState, dir: Dir1D) -> Res {
        // navigate product field structure; ignore unchosen variant options.
        match menu.tree {
            MenuTree::Blank(_) => Err("no subtrees".to_string()),
            MenuTree::Product(ref trees) => {
                let mut trees = trees.clone();
                if trees.len() > 0 {
                    match dir {
                        Dir1D::Forward => {
                            trees.rotate_left(1);
                            let (label, tree, tree_t) = trees.pop().unwrap();
                            menu.tree = tree;
                            menu.tree_typ = tree_t;
                            menu.ctx = MenuCtx::Product(Box::new(LabelSelect {
                                before: vec![],
                                ctx: menu.ctx.clone(),
                                label: label,
                                after: trees,
                            }));
                            Ok(())
                        }
                        Dir1D::Backward => {
                            let (label, tree, tree_t) = trees.pop().unwrap();
                            menu.tree = tree;
                            menu.tree_typ = tree_t;
                            menu.ctx = MenuCtx::Product(Box::new(LabelSelect {
                                before: trees,
                                ctx: menu.ctx.clone(),
                                label: label,
                                after: vec![],
                            }));
                            Ok(())
                        }
                    }
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
            MenuCtx::Product(ref ctx) => unimplemented!(),
            _ => unimplemented!(),
        }
    }

    pub fn prev_sibling(menu: &mut MenuState) -> Res {
        unimplemented!()
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
            (&MenuTree::Variant(ref arms1), &MenuTree::Variant(ref arms2)) => unimplemented!(),
            (&MenuTree::Product(ref fields1), &MenuTree::Product(ref fields2)) => {
                let mut fields3 = vec![];
                for ((l1, t1, tt1), (l2, t2, tt2)) in fields1.iter().zip(fields2.iter()) {
                    assert_eq!(l1, l2);
                    assert_eq!(tt1, tt2);
                    let t3 = tree_union(t1, t2);
                    fields3.push((l1.clone(), t3, tt1.clone()));
                }
                MenuTree::Product(fields3)
            }
            (_, _) => unimplemented!(),
        }
    }

    pub fn auto_fill(typ: &MenuType, depth: usize) -> MenuTree {
        if depth == 0 {
            MenuTree::Blank(typ.clone())
        } else {
            match typ {
                &MenuType::Prim(PrimType::Nat) => MenuTree::Nat(0),
                &MenuType::Prim(PrimType::Text) => MenuTree::Text("".to_string()),
                &MenuType::Prim(PrimType::Bool) => MenuTree::Bool(false),
                &MenuType::Variant(ref labtyps) => {
                    let after_choices: Vec<(Label, MenuTree, MenuType)> = labtyps
                        .iter()
                        .map(|(l, lt)| (l.clone(), auto_fill(lt, depth - 1), lt.clone()))
                        .collect();
                    MenuTree::Variant(Box::new(LabelChoice {
                        before: vec![],
                        choice: None,
                        after: after_choices,
                    }))
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
}

pub mod io {
    use super::{EditCommand, Label, MenuCtx, MenuState, MenuTree};
    use render::Render;
    use types::event::Event;
    use types::{
        lang::{Dir1D, Dir2D, Name},
        render::{Color, Dim, Elms, Fill},
    };

    pub fn edit_commands_of_event(event: &Event) -> Result<Vec<EditCommand>, ()> {
        match event {
            &Event::Quit { .. } => Err(()),
            &Event::KeyDown(ref kei) => match kei.key.as_str() {
                "Escape" => Err(()),
                " " => Ok(vec![]),
                "Tab" => Ok(vec![EditCommand::AutoFill]),
                "ArrowLeft" => Ok(vec![EditCommand::PrevTree]),
                "ArrowRight" => Ok(vec![EditCommand::NextTree]),
                "ArrowUp" => Ok(vec![EditCommand::PrevBlank]),
                "ArrowDown" => Ok(vec![EditCommand::NextBlank]),
                _ => Ok(vec![]),
            },
            _ => Ok(vec![]),
        }
    }

    pub fn render_elms(menu: &MenuState) -> Result<Elms, String> {
        use crate::render::{FlowAtts, FrameType, TextAtts};

        fn black_fill() -> Fill {
            Fill::Closed(Color::RGB(0, 0, 0))
        }

        // eventually we get these atts from
        //  some environment-determined settings
        fn meta_atts() -> TextAtts {
            TextAtts {
                zoom: 2,
                fg_fill: Fill::Closed(Color::RGB(255, 200, 255)),
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
        fn text_atts() -> TextAtts {
            TextAtts {
                zoom: 1,
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
                zoom: 1,
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
            let mut next_ctx = None;
            r.begin(&Name::Void, FrameType::Flow(tree_flow()));
            r.fill(black_fill());
            match ctx {
                &MenuCtx::Root => {
                    r.str("/", &text_atts());
                }
                &MenuCtx::Product(ref sel) => {
                    next_ctx = Some(sel.ctx.clone());
                    r.begin(&Name::Void, FrameType::Flow(sub_flow()));
                    for (l, t, ty) in sel.before.iter() {
                        begin_item(r);
                        render_product_label(l, r);
                        render_tree(t, r);
                        r.end();
                    }
                    {
                        begin_item(r);
                        render_product_label(&sel.label, r);
                        r.str(" ...", &text_atts());
                        r.end();
                    }
                    for (l, t, ty) in sel.after.iter() {
                        begin_item(r);
                        render_product_label(&l, r);
                        render_tree(t, r);
                        r.end();
                    }
                    r.end();
                }
                &MenuCtx::Variant(ref sel) => {
                    r.begin(&Name::Void, FrameType::Flow(sub_flow()));
                    for (l, t, ty) in sel.before.iter() {
                        begin_item(r);
                        render_variant_label(&l, r);
                        render_tree(t, r);
                        r.end()
                    }
                    {
                        begin_item(r);
                        render_variant_label(&sel.label, r);
                        r.str(" ...", &text_atts());
                        render_ctx(&*ctx, r);
                        r.end();
                        next_ctx = Some(sel.ctx.clone());
                    }
                    for (l, t, ty) in sel.after.iter() {
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
                info!("--- context continues... ---");
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
                    for (l, t, ty) in ch.before.iter() {
                        begin_item(r);
                        render_variant_label(l, r);
                        render_tree(t, r);
                        r.end()
                    }
                    if let Some((ref l, ref tree, ref _tree_t)) = ch.choice {
                        begin_item(r);
                        render_variant_label(l, r);
                        render_tree(&tree, r);
                        r.end();
                    };
                    for (l, t, ty) in ch.after.iter() {
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
                //&MenuTree::Blank(ref typ) => r.text(&format!("__{:?}__", typ), &blank_atts()),
                &MenuTree::Blank(ref typ) => r.text(&format!("___"), &blank_atts()),
                &MenuTree::Nat(n) => r.text(&format!("{}", n), &text_atts()),
                &MenuTree::Bool(b) => r.text(&format!("{}", b), &text_atts()),
                &MenuTree::Text(ref t) => r.text(t, &text_atts()),
            };
            //info!("render_tree({:?}): end.", tree);
            r.end();
        };
        info!("render_menu_begin");
        let mut r = Render::new();
        r.begin(&Name::Void, FrameType::Flow(sub_flow()));
        r.fill(black_fill());
        r.str("menu{", &meta_atts());
        {
            r.begin(&Name::Void, FrameType::Flow(tree_flow()));
            r.str("ctx=", &meta_atts());
            r.begin(&Name::Void, FrameType::Flow(ctx_flow()));
            {
                render_ctx(&menu.ctx, &mut r);
            }
            r.end();
            r.end();

            r.begin(&Name::Void, FrameType::Flow(tree_flow()));
            r.str("tree=", &meta_atts());
            r.begin(&Name::Void, FrameType::Flow(tree_flow()));
            {
                render_tree(&menu.tree, &mut r);
            }
            r.end();
            r.end();
        }
        r.str("}", &meta_atts());
        r.end();
        info!("render_menu_end");
        Ok(r.into_elms())
    }
}
