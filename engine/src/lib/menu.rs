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
    Unit,
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
    Unit,
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
    Unit,
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
    GotoRoot,       // ---?
    AutoFill,       // Tab
    NextTree,       // ArrowRight
    PrevTree,       // ArrowLeft
    NextBlank,      // ---?
    PrevBlank,      // ---?
    NextVariant,    // ArrowRight
    PrevVariant,    // ArrowLefet
    AcceptVariant,  // Enter
    VecInsertBlank, // ---?
    VecInsertAuto,  // ---?
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
    pub root_typ: Rc<MenuType>, // invariant: root_typ = unfocus(ctx, tree_typ);
    pub ctx: MenuCtx,           // invariant: see typ.
    pub tree: MenuTree,         // invariant: tree has type tree_typ.
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
                    root_typ: Rc::new(typ.clone()),
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
                let tree = auto_fill(&menu.tree_typ, 1);
                drop(tree_union(&menu.tree, &tree));
                menu.tree = tree;
                Ok(())
            }
            &EditCommand::NextTree => next_tree(menu).map(|_| ()),
            &EditCommand::PrevTree => prev_tree(menu).map(|_| ()),

            &EditCommand::NextVariant => cycle_variant(menu, Dir1D::Forward),
            &EditCommand::PrevVariant => cycle_variant(menu, Dir1D::Backward),
            &EditCommand::AcceptVariant => {
                assert_tree_tag(&menu.tree, &Tag::Variant)?;
                ascend(menu)
            }

            &EditCommand::NextBlank => next_blank(menu).map(|_| ()),
            &EditCommand::PrevBlank => prev_blank(menu).map(|_| ()),

            &EditCommand::VecInsertBlank => {
                assert_tree_tag(&menu.tree, &Tag::Vec)?;
                unimplemented!()
            }
            &EditCommand::VecInsertAuto => {
                assert_tree_tag(&menu.tree, &Tag::Vec)?;
                state_eval_command(menu, &EditCommand::VecInsertBlank)?;
                state_eval_command(menu, &EditCommand::AutoFill)
            }
            _ => unimplemented!(),
        }
    }

    pub fn assert_tree_tag(tree: &MenuTree, tag: &Tag) -> Res {
        // to do
        Ok(())
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
        match menu.ctx.clone() {
            MenuCtx::Root => Err("cannot ascend: already at root".to_string()),
            MenuCtx::Product(mut sel) => {
                let mut arms = sel.before;
                arms.push((sel.label, menu.tree.clone(), menu.tree_typ.clone()));
                arms.append(&mut sel.after);
                let fields: Vec<(Label, MenuType)> = arms
                    .iter()
                    .map(|(l, _, t)| (l.clone(), t.clone()))
                    .collect();
                menu.tree = MenuTree::Product(arms);
                menu.tree_typ = MenuType::Product(fields);
                menu.ctx = sel.ctx;
                Ok(())
            }
            MenuCtx::Variant(sel) => {
                let fields: Vec<(Label, MenuType)> = {
                    let mut sel = sel.clone();
                    let mut arms = sel.before;
                    arms.push((sel.label, menu.tree.clone(), menu.tree_typ.clone()));
                    arms.append(&mut sel.after);
                    arms.iter()
                        .map(|(l, _, t)| (l.clone(), t.clone()))
                        .collect()
                };
                menu.tree = MenuTree::Variant(Box::new(LabelChoice {
                    before: sel.before,
                    choice: Some((sel.label, menu.tree.clone(), menu.tree_typ.clone())),
                    after: sel.after,
                }));
                menu.tree_typ = MenuType::Variant(fields);
                menu.ctx = sel.ctx;
                Ok(())
            }
            MenuCtx::Option(flag, menu) => unimplemented!(),
            MenuCtx::Vec(sel) => unimplemented!(),
            MenuCtx::Tup(sel) => unimplemented!(),
        }
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
            MenuTree::Variant(ref trees) => {
                let trees = trees.clone();
                match trees.choice {
                    Some((label, tree, tree_t)) => match dir {
                        Dir1D::Forward | Dir1D::Backward => {
                            menu.tree = tree;
                            menu.tree_typ = tree_t;
                            menu.ctx = MenuCtx::Variant(Box::new(LabelSelect {
                                before: trees.before.clone(),
                                ctx: menu.ctx.clone(),
                                label: label,
                                after: trees.after.clone(),
                            }));
                            Ok(())
                        }
                    },
                    None => Err("no choice subtree".to_string()),
                }
            }
            _ => {
                // to do
                Err("not implemented".to_string())
            }
        }
    }

    pub fn cycle_variant(menu: &mut MenuState, dir: Dir1D) -> Res {
        match menu.tree {
            MenuTree::Variant(ref arms) => {
                let mut arms = arms.clone();
                match dir {
                    Dir1D::Forward => {
                        if arms.after.len() > 0 {
                            arms.after.rotate_left(1);
                            let (label, tree, tree_t) = arms.after.pop().unwrap();
                            if let Some(ch) = arms.choice {
                                arms.before.push(ch);
                            }
                            arms.choice = Some((label, tree, tree_t));
                            menu.tree = MenuTree::Variant(arms);
                            Ok(())
                        } else {
                            arms.after = arms.before;
                            arms.before = vec![];
                            if let Some(ch) = arms.choice {
                                arms.after.push(ch);
                                arms.choice = None;
                            }
                            menu.tree = MenuTree::Variant(arms);
                            Ok(())
                        }
                    }
                    Dir1D::Backward => {
                        if arms.before.len() > 0 {
                            let (label, tree, tree_t) = arms.before.pop().unwrap();
                            if let Some(ch) = arms.choice {
                                let mut after_ch = arms.after;
                                arms.after = vec![ch];
                                arms.after.append(&mut after_ch);
                            }
                            arms.choice = Some((label, tree, tree_t));
                            menu.tree = MenuTree::Variant(arms);
                            Ok(())
                        } else {
                            if let Some(ch) = arms.choice {
                                arms.before = vec![ch];
                                arms.choice = None;
                            }
                            arms.before.append(&mut arms.after);
                            arms.after = vec![];
                            menu.tree = MenuTree::Variant(arms);
                            Ok(())
                        }
                    }
                }
            }
            _ => Err("expected tree to be a variant".to_string()),
        }
    }

    pub fn next_sibling(menu: &mut MenuState) -> Res {
        match menu.ctx.clone() {
            MenuCtx::Product(mut sel) => {
                sel.before
                    .push((sel.label, menu.tree.clone(), menu.tree_typ.clone()));
                if sel.after.len() > 0 {
                    sel.after.rotate_left(1);
                    let (label, tree, tree_typ) = sel.after.pop().unwrap();
                    sel.label = label;
                    menu.tree = tree;
                    menu.tree_typ = tree_typ;
                    menu.ctx = MenuCtx::Product(sel);
                    Ok(())
                } else {
                    ascend(menu)?;
                    next_sibling(menu)
                }
            }
            MenuCtx::Root => Ok(()),
            MenuCtx::Variant(_) => {
                ascend(menu)?;
                next_sibling(menu)
            }
            _ => {
                error!("{:?}", menu.ctx);
                Ok(())
            }
        }
    }

    pub fn prev_sibling(menu: &mut MenuState) -> Res {
        Err("unimplemented".to_string())
    }

    // to do -- change name to `tree_update`? -- prefers second tree when they are each non-blank and disagree
    pub fn tree_union(tree1: &MenuTree, tree2: &MenuTree) -> MenuTree {
        // todo -- assert that the blanks' types agree
        match (tree1, tree2) {
            (&MenuTree::Blank(_), _) => tree2.clone(),
            (_, &MenuTree::Blank(_)) => tree1.clone(),
            (&MenuTree::Variant(ref arms1), &MenuTree::Variant(ref arms2)) => {
                // to do
                unimplemented!()
            }
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
            // to do -- handle other cases by preferring second tree?
            (_, _) => unimplemented!(),
        }
    }

    pub fn auto_fill(typ: &MenuType, depth: usize) -> MenuTree {
        if depth == 0 {
            MenuTree::Blank(typ.clone())
        } else {
            match typ {
                &MenuType::Prim(PrimType::Unit) => MenuTree::Unit,
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
                "Tab" => Ok(vec![EditCommand::AutoFill]),

                "ArrowLeft" => Ok(vec![EditCommand::PrevTree]),
                "ArrowRight" => Ok(vec![EditCommand::NextTree]),

                "ArrowUp" => Ok(vec![EditCommand::PrevVariant]),
                "ArrowDown" => Ok(vec![EditCommand::NextVariant]),
                "Enter" => Ok(vec![EditCommand::AcceptVariant]),

                _ => Ok(vec![]),
            },
            _ => Ok(vec![]),
        }
    }

    pub fn render_elms(menu: &MenuState) -> Result<Elms, String> {
        use crate::render::{FlowAtts, FrameType, TextAtts};

        fn black_fill() -> Fill {
            //Fill::Closed(Color::RGB(0, 0, 0))
            Fill::None
        }

        fn ctx_box_fill() -> Fill {
            Fill::Open(Color::RGB(255, 100, 255), 1)
        }

        fn tree_box_fill() -> Fill {
            Fill::Open(Color::RGB(100, 255, 100), 1)
        }

        fn text_zoom() -> usize {
            2
        }
        fn glyph_padding() -> usize {
            1
        }
        fn horz_flow() -> FlowAtts {
            FlowAtts {
                dir: Dir2D::Right,
                padding: 2,
            }
        };
        fn vert_flow() -> FlowAtts {
            FlowAtts {
                dir: Dir2D::Down,
                padding: 2,
            }
        };

        // eventually we get these atts from
        //  some environment-determined settings
        fn meta_atts() -> TextAtts {
            TextAtts {
                zoom: text_zoom(),
                fg_fill: Fill::Closed(Color::RGB(255, 200, 255)),
                bg_fill: Fill::None,
                glyph_dim: Dim {
                    width: 5,
                    height: 5,
                },
                glyph_flow: FlowAtts {
                    dir: Dir2D::Right,
                    padding: glyph_padding(),
                },
            }
        };
        fn kw_atts() -> TextAtts {
            TextAtts {
                zoom: text_zoom(),
                fg_fill: Fill::Closed(Color::RGB(255, 255, 255)),
                bg_fill: Fill::None,
                glyph_dim: Dim {
                    width: 5,
                    height: 5,
                },
                glyph_flow: FlowAtts {
                    dir: Dir2D::Right,
                    padding: glyph_padding(),
                },
            }
        };
        fn text_atts() -> TextAtts {
            TextAtts {
                zoom: text_zoom(),
                fg_fill: Fill::Closed(Color::RGB(200, 200, 200)),
                bg_fill: Fill::None,
                glyph_dim: Dim {
                    width: 5,
                    height: 5,
                },
                glyph_flow: FlowAtts {
                    dir: Dir2D::Right,
                    padding: glyph_padding(),
                },
            }
        };
        fn blank_atts() -> TextAtts {
            TextAtts {
                zoom: text_zoom(),
                fg_fill: Fill::Closed(Color::RGB(255, 200, 200)),
                bg_fill: Fill::Closed(Color::RGB(100, 0, 0)),
                glyph_dim: Dim {
                    width: 5,
                    height: 5,
                },
                glyph_flow: FlowAtts {
                    dir: Dir2D::Right,
                    padding: glyph_padding(),
                },
            }
        };
        fn pad_atts() -> TextAtts {
            TextAtts {
                zoom: 1,
                fg_fill: Fill::None,
                bg_fill: Fill::None,
                glyph_dim: Dim {
                    width: 5,
                    height: 5,
                },
                glyph_flow: FlowAtts {
                    dir: Dir2D::Right,
                    padding: 0,
                },
            }
        };
        fn cursor_atts() -> TextAtts {
            TextAtts {
                zoom: text_zoom(),
                fg_fill: Fill::Closed(Color::RGB(255, 255, 255)),
                bg_fill: Fill::None,
                glyph_dim: Dim {
                    width: 5,
                    height: 5,
                },
                glyph_flow: FlowAtts {
                    dir: Dir2D::Right,
                    padding: 1,
                },
            }
        };

        fn render_choice_label(label: &Label, r: &mut Render) {
            r.str("#", &kw_atts());
            r.name(label, &text_atts());
            r.str("=", &kw_atts());
        }

        fn render_product_label(label: &Label, r: &mut Render) {
            r.str("*", &kw_atts());
            r.name(label, &text_atts());
            r.str("=", &kw_atts());
        }

        fn render_variant_label(is_chosen: bool, label: &Label, r: &mut Render) {
            if is_chosen {
                r.str("*", &cursor_atts());
            } else {
                r.str(" ", &cursor_atts());
            }
            r.str("#", &kw_atts());
            r.name(label, &text_atts());
            r.str("=", &kw_atts());
        }

        fn begin_item(r: &mut Render) {
            r.begin(&Name::Void, FrameType::Flow(horz_flow()))
        }

        fn render_ctx(ctx: &MenuCtx, show_detailed: bool, r_out: &mut Render, r_tree: Render) {
            let mut next_ctx = None;
            let mut r = Render::new();

            r.begin(&Name::Void, FrameType::Flow(horz_flow()));
            r.fill(ctx_box_fill());

            match ctx {
                &MenuCtx::Root => {
                    drop(r);
                    r_out.nest(&Name::Void, r_tree);
                    return;
                }
                &MenuCtx::Product(ref sel) => {
                    next_ctx = Some(sel.ctx.clone());
                    r.begin(&Name::Void, FrameType::Flow(vert_flow()));
                    for (l, t, ty) in sel.before.iter() {
                        begin_item(&mut r);
                        render_product_label(l, &mut r);
                        render_tree(t, false, &ctx_box_fill(), &mut r);
                        r.end();
                    }
                    {
                        begin_item(&mut r);
                        render_product_label(&sel.label, &mut r);
                        r.nest(&Name::Void, r_tree);
                        r.end();
                    }
                    for (l, t, ty) in sel.after.iter() {
                        begin_item(&mut r);
                        render_product_label(&l, &mut r);
                        render_tree(t, false, &ctx_box_fill(), &mut r);
                        r.end();
                    }
                    r.end();
                }
                &MenuCtx::Variant(ref sel) => {
                    next_ctx = Some(sel.ctx.clone());
                    begin_item(&mut r);
                    render_choice_label(&sel.label, &mut r);
                    r.nest(&Name::Void, r_tree);
                    r.end();
                }
                &MenuCtx::Option(flag, ref body) => unimplemented!(),
                &MenuCtx::Vec(ref ch) => unimplemented!(),
                &MenuCtx::Tup(ref ch) => unimplemented!(),
            };
            r.end();
            // continue rendering the rest of the context, in whatever flow we are using for that purpose.
            if let Some(ctx) = next_ctx {
                render_ctx(&ctx, false, r_out, r)
            } else {
                //info!("context end: root.");
            };
        };

        fn render_tree(tree: &MenuTree, show_detailed: bool, box_fill: &Fill, r: &mut Render) {
            r.begin(&Name::Void, FrameType::Flow(horz_flow()));
            r.fill(box_fill.clone());
            match tree {
                &MenuTree::Product(ref fields) => {
                    r.begin(&Name::Void, FrameType::Flow(vert_flow()));
                    for (l, t, ty) in fields.iter() {
                        begin_item(r);
                        render_product_label(l, r);
                        render_tree(t, false, box_fill, r);
                        r.end()
                    }
                    r.end()
                }
                &MenuTree::Variant(ref ch) => {
                    r.begin(&Name::Void, FrameType::Flow(vert_flow()));

                    begin_item(r);
                    if let Some((ref label, ref tree, _)) = ch.choice {
                        render_choice_label(&label, r);
                        render_tree(tree, false, box_fill, r);
                    } else {
                        r.text(&format!("___"), &blank_atts());
                    };
                    r.end();
                    if show_detailed {
                        for (l, t, ty) in ch.before.iter() {
                            begin_item(r);
                            render_variant_label(false, l, r);
                            render_tree(t, false, box_fill, r);
                            r.end()
                        }
                        if let Some((ref l, ref tree, ref _tree_t)) = ch.choice {
                            begin_item(r);
                            render_variant_label(true, l, r);
                            render_tree(&tree, false, box_fill, r);
                            r.end();
                        };
                        for (l, t, ty) in ch.after.iter() {
                            begin_item(r);
                            render_variant_label(false, l, r);
                            render_tree(t, false, box_fill, r);
                            r.end()
                        }
                    }
                    r.end()
                }
                &MenuTree::Option(flag, ref tree, ref typ) => {
                    if flag {
                        r.str("?", &text_atts())
                    };
                    render_tree(&*tree, false, box_fill, r)
                }
                &MenuTree::Vec(ref trees, ref _typ) => {
                    r.begin(&Name::Void, FrameType::Flow(horz_flow()));
                    for tree in trees.iter() {
                        render_tree(tree, false, box_fill, r)
                    }
                    r.end();
                }
                &MenuTree::Tup(ref trees) => {
                    r.begin(&Name::Void, FrameType::Flow(horz_flow()));
                    for (tree, _typ) in trees.iter() {
                        render_tree(tree, false, box_fill, r)
                    }
                    r.end();
                }
                //&MenuTree::Blank(ref typ) => r.text(&format!("__{:?}__", typ), &blank_atts()),
                &MenuTree::Blank(ref typ) => r.text(&format!("___"), &blank_atts()),
                &MenuTree::Nat(n) => r.text(&format!("{}", n), &text_atts()),
                &MenuTree::Bool(b) => r.text(&format!("{}", b), &text_atts()),
                &MenuTree::Text(ref t) => r.text(&format!("{:?}", t), &text_atts()),
                &MenuTree::Unit => r.str("()", &text_atts()),
            };
            r.end();
        };
        let mut r = Render::new();
        r.begin(&Name::Void, FrameType::Flow(vert_flow()));
        if true {
            r.str("hello world!", &text_atts());
            r.str(" please, enter a value to submit:", &text_atts());
            r.str(
                " (keys: Tab, Right, Down, Up, Left, Enter, Esc,",
                &text_atts(),
            );

            let mut r_tree = {
                let mut r_tree = Render::new();
                render_tree(&menu.tree, true, &tree_box_fill(), &mut r_tree);
                r_tree
            };
            render_ctx(&menu.ctx, false, &mut r, r_tree);
        }
        r.end();
        Ok(r.into_elms())
    }
}
