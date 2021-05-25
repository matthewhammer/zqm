use serde::{Deserialize, Serialize};
use std::rc::Rc;
use types::lang::{Atom, Dir1D, Name};

pub type Text = String;
pub type Nat = usize; // todo -- use a bignum rep
pub type Int = isize; // todo -- use a bignum rep
pub type Label = Name;

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum MenuType {
    Prim(PrimType),
    Variant(Vec<(Label, MenuType)>),
    Product(Vec<(Label, MenuType)>),
    Option(Box<MenuType>),
    Vec(Box<MenuType>),
    Tup(Vec<MenuType>),
    Var(Name),
    Func(FuncType),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum PrimType {
    Null,
    Unit,
    Nat,
    Int,
    Text,
    Bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct FuncType {
    pub args: Vec<MenuType>,
    pub rets: Vec<MenuType>,
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
    Int(Int),
    Text(Text),
    Bool(bool),
    Unit,
    Null,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct LabelSelect {
    pub before: Vec<(Label, MenuTree, MenuType)>,
    pub ctx: MenuCtx,
    pub label: Label,
    pub after: Vec<(Label, MenuTree, MenuType)>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct PosSelect {
    pub before: Vec<(MenuTree, MenuType)>,
    pub ctx: MenuCtx,
    pub after: Vec<(MenuTree, MenuType)>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct LabelChoice {
    pub before: Vec<(Label, MenuTree, MenuType)>,
    pub choice: Option<(Label, MenuTree, MenuType)>,
    pub after: Vec<(Label, MenuTree, MenuType)>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum Error {
    MenuTypeMismatch(MenuType, Tag), // found vs expected
    Blank(Tag),                      // found blank vs expected completed 'tag'
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Tag {
    Root,
    Prim(PrimType),
    Variant,
    Product,
    Option,
    Vec,
    Tup,
    Blank,
    Var,
    Func,
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
    Commit,
    Descend,
    Ascend,
    PrevSibling,
    NextSibling,
    GotoRoot,
    AutoFill,
    Clear,
    NextTree,
    PrevTree,
    NextBlank,
    PrevBlank,
    NextVariant,
    PrevVariant,
    AcceptVariant,
    VecInsertBlank,
    VecInsertAuto,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum Command {
    Init(InitCommand),
    Auto(AutoCommand),
    Edit(EditCommand),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum MenuCtx {
    Root(MenuType),
    Tup(Box<PosSelect>),
    Product(Box<LabelSelect>),
    Variant(Box<LabelSelect>),
    Option(bool, Box<MenuCtx>),
    Vec(Box<PosSelect>),
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

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum Halt {
    Commit(MenuTree),
    Message(String),
}

pub mod semantics {
    use super::*;

    pub type Res = Result<(), Halt>;

    pub fn editor_eval(menu: &mut Editor, command: &Command) -> Res {
        trace!("editor_eval({:?}) begin", command);
        let res = match command {
            Command::Init(InitCommand::Default(ref default_choice, ref typ)) => {
                menu.state = Some(MenuState {
                    root_typ: Rc::new(typ.clone()),
                    ctx: MenuCtx::Root(typ.clone()),
                    tree: default_choice.clone(),
                    tree_typ: typ.clone(),
                });
                Ok(())
            }
            Command::Edit(ref c) => match menu.state {
                None => Err(Halt::Message("Invalid editor state".to_string())),
                Some(ref mut st) => state_eval_command(st, c),
            },
            Command::Auto(ref _c) => unimplemented!(),
        };
        debug!("editor_eval({:?}) ==> {:?}", command, res);
        menu.history.push(command.clone());
        res
    }

    pub fn state_eval_command(menu: &mut MenuState, command: &EditCommand) -> Res {
        match command {
            &EditCommand::Commit => {
                goto_root(menu)?;
                Err(Halt::Commit(menu.tree.clone()))
            }
            &EditCommand::GotoRoot => goto_root(menu),
            &EditCommand::AutoFill => {
                let tree = auto_fill(&menu.tree_typ, 1);
                drop(tree_union(&menu.tree, &tree));
                menu.tree = tree;
                Ok(())
            }
            &EditCommand::Clear => {
                menu.tree = MenuTree::Blank(menu.tree_typ.clone());
                Ok(())
            }
            &EditCommand::NextTree => next_tree(menu).map(|_| ()),
            &EditCommand::PrevTree => prev_tree(menu).map(|_| ()),

            &EditCommand::Ascend => ascend(menu).map(|_| ()),
            &EditCommand::Descend => descend(menu, Dir1D::Forward).map(|_| ()),

            &EditCommand::NextSibling => next_sibling(menu).map(|_| ()),
            &EditCommand::PrevSibling => prev_sibling(menu).map(|_| ()),

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
        }
    }

    pub fn tree_tag(tree: &MenuTree) -> Tag {
        match tree {
            MenuTree::Product(_) => Tag::Product,
            MenuTree::Variant(_) => Tag::Variant,
            MenuTree::Option(_, _, _) => Tag::Option,
            MenuTree::Vec(_, _) => Tag::Vec,
            MenuTree::Tup(_) => Tag::Tup,
            MenuTree::Blank(_) => Tag::Blank,
            MenuTree::Nat(_) => Tag::Prim(PrimType::Nat),
            MenuTree::Int(_) => Tag::Prim(PrimType::Int),
            MenuTree::Bool(_) => Tag::Prim(PrimType::Bool),
            MenuTree::Text(_) => Tag::Prim(PrimType::Text),
            MenuTree::Unit => Tag::Prim(PrimType::Unit),
            MenuTree::Null => Tag::Prim(PrimType::Null),
        }
    }

    pub fn ctx_tag(ctx: &MenuCtx) -> Tag {
        match ctx {
            MenuCtx::Root(typ) => Tag::Root,
            MenuCtx::Product(_) => Tag::Product,
            MenuCtx::Variant(_) => Tag::Variant,
            MenuCtx::Option(_, _) => Tag::Option,
            MenuCtx::Tup(_) => Tag::Tup,
            MenuCtx::Vec(_) => Tag::Vec,
        }
    }

    pub fn typ_tag(typ: &MenuType) -> Tag {
        match typ {
            MenuType::Prim(t) => Tag::Prim(t.clone()),
            MenuType::Product(_) => Tag::Product,
            MenuType::Variant(_) => Tag::Variant,
            MenuType::Option(_) => Tag::Option,
            MenuType::Tup(_) => Tag::Tup,
            MenuType::Vec(_) => Tag::Vec,
            MenuType::Var(_) => Tag::Var,
            MenuType::Func(_) => Tag::Func,
        }
    }

    pub fn assert_tree_tag(tree: &MenuTree, tag: &Tag) -> Res {
        let tt = tree_tag(tree);
        if &tt == tag {
            Ok(())
        } else {
            Err(Halt::Message(format!(
                "expected {:?} but found {:?}: {:?}",
                tag, tt, tree
            )))
        }
    }

    pub fn goto_root(menu: &mut MenuState) -> Res {
        match menu.ctx {
            MenuCtx::Root(_) => Ok(()),
            _ => {
                ascend(menu)?;
                goto_root(menu)
            }
        }
    }

    pub fn next_blank(menu: &mut MenuState) -> Result<MenuType, Halt> {
        match menu.tree {
            MenuTree::Blank(ref t) => Ok(t.clone()),
            _ => {
                next_tree(menu)?;
                next_blank(menu)
            }
        }
    }

    pub fn prev_blank(menu: &mut MenuState) -> Result<MenuType, Halt> {
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
            MenuCtx::Root(_) => Err(Halt::Message("cannot ascend: already at root".to_string())),
            MenuCtx::Tup(sel) => {
                let mut arms: Vec<(MenuTree, MenuType)> = sel
                    .before
                    .iter()
                    .map(|(t, tt)| (t.clone(), tt.clone()))
                    .collect();
                arms.push((menu.tree.clone(), menu.tree_typ.clone()));
                let mut after = sel
                    .after
                    .iter()
                    .map(|(t, tt)| (t.clone(), tt.clone()))
                    .collect();
                arms.append(&mut after);
                let fields: Vec<MenuType> = arms.iter().map(|(_, t)| t.clone()).collect();
                menu.tree = MenuTree::Tup(arms);
                menu.tree_typ = MenuType::Tup(fields);
                menu.ctx = sel.ctx;
                Ok(())
            }
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
            MenuCtx::Option(_flag, _menu) => unimplemented!(),
            MenuCtx::Vec(_sel) => unimplemented!(),
        }
    }

    pub fn descend(menu: &mut MenuState, dir: Dir1D) -> Res {
        // navigate product field structure; ignore unchosen variant options.
        match menu.tree {
            MenuTree::Blank(_) => Err(Halt::Message("no subtrees".to_string())),
            MenuTree::Tup(ref trees) => {
                let mut trees = trees.clone();
                if trees.len() > 0 {
                    match dir {
                        Dir1D::Forward => {
                            trees.rotate_left(1);
                            let (tree, tree_t) = trees.pop().unwrap();
                            menu.tree = tree;
                            menu.tree_typ = tree_t;
                            menu.ctx = MenuCtx::Tup(Box::new(PosSelect {
                                before: vec![],
                                ctx: menu.ctx.clone(),
                                after: trees,
                            }));
                            Ok(())
                        }
                        Dir1D::Backward => {
                            let (tree, tree_t) = trees.pop().unwrap();
                            menu.tree = tree;
                            menu.tree_typ = tree_t;
                            menu.ctx = MenuCtx::Tup(Box::new(PosSelect {
                                before: trees,
                                ctx: menu.ctx.clone(),
                                after: vec![],
                            }));
                            Ok(())
                        }
                    }
                } else {
                    Err(Halt::Message("no subtrees".to_string()))
                }
            }
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
                    Err(Halt::Message("no subtrees".to_string()))
                }
            }
            MenuTree::Nat(ref n) => {
                // to do -- move this logic
                menu.tree = MenuTree::Nat(*n + 1);
                Ok(())
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
                    None => Err(Halt::Message("no choice subtree".to_string())),
                }
            }
            _ => {
                // to do
                Err(Halt::Message("not implemented".to_string()))
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
            _ => Err(Halt::Message("expected tree to be a variant".to_string())),
        }
    }

    pub fn next_sibling(menu: &mut MenuState) -> Res {
        match menu.ctx.clone() {
            MenuCtx::Tup(mut sel) => {
                sel.before.push((menu.tree.clone(), menu.tree_typ.clone()));
                if sel.after.len() > 0 {
                    sel.after.rotate_left(1);
                    let (tree, tree_typ) = sel.after.pop().unwrap();
                    menu.tree = tree;
                    menu.tree_typ = tree_typ;
                    menu.ctx = MenuCtx::Tup(sel);
                    Ok(())
                } else {
                    ascend(menu)?;
                    descend(menu, Dir1D::Forward)
                }
            }
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
                    descend(menu, Dir1D::Forward)
                }
            }
            MenuCtx::Root(_) => Ok(()),
            MenuCtx::Variant(_) => next_sibling(menu),
            _ => {
                error!("{:?}", menu.ctx);
                Ok(())
            }
        }
    }

    pub fn prev_sibling(menu: &mut MenuState) -> Res {
        match menu.ctx.clone() {
            MenuCtx::Product(mut sel) => {
                sel.after
                    .push((sel.label, menu.tree.clone(), menu.tree_typ.clone()));
                sel.after.rotate_left(1);
                if sel.before.len() > 0 {
                    let (label, tree, tree_typ) = sel.before.pop().unwrap();
                    sel.label = label;
                    menu.tree = tree;
                    menu.tree_typ = tree_typ;
                    menu.ctx = MenuCtx::Product(sel);
                    Ok(())
                } else {
                    ascend(menu)?;
                    descend(menu, Dir1D::Backward)
                }
            }
            MenuCtx::Tup(mut sel) => {
                sel.after.push((menu.tree.clone(), menu.tree_typ.clone()));
                sel.after.rotate_left(1);
                if sel.before.len() > 0 {
                    let (tree, tree_typ) = sel.before.pop().unwrap();
                    menu.tree = tree;
                    menu.tree_typ = tree_typ;
                    menu.ctx = MenuCtx::Tup(sel);
                    Ok(())
                } else {
                    ascend(menu)?;
                    descend(menu, Dir1D::Backward)
                }
            }
            MenuCtx::Root(_) => Ok(()),
            MenuCtx::Variant(_) => {
                ascend(menu)?;
                prev_sibling(menu)
            }
            _ => {
                error!("{:?}", menu.ctx);
                Ok(())
            }
        }
    }

    // to do -- change name to `tree_update`? -- prefers second tree when they are each non-blank and disagree
    pub fn tree_union(tree1: &MenuTree, tree2: &MenuTree) -> MenuTree {
        // todo -- assert that the blanks' types agree
        match (tree1, tree2) {
            (&MenuTree::Blank(_), _) => tree2.clone(),
            (_, &MenuTree::Blank(_)) => tree1.clone(),
            (&MenuTree::Variant(ref _arms1), &MenuTree::Variant(ref _arms2)) => {
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
                &MenuType::Prim(PrimType::Null) => MenuTree::Null,
                &MenuType::Prim(PrimType::Nat) => MenuTree::Nat(0),
                &MenuType::Prim(PrimType::Int) => MenuTree::Int(0),
                &MenuType::Prim(PrimType::Text) => MenuTree::Text("".to_string()),
                &MenuType::Prim(PrimType::Bool) => MenuTree::Bool(false),
                &MenuType::Var(ref n) => MenuTree::Blank(typ.clone()),
                &MenuType::Func(ref f) => MenuTree::Blank(typ.clone()),
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
    use super::{
        semantics::{ctx_tag, tree_tag},
        EditCommand, Label, MenuCtx, MenuState, MenuTree, Tag,
    };
    use render::Render;
    use types::event::Event;
    use types::{
        lang::{Dir2D, Name},
        render::{Color, Dim, Elms, Fill},
    };

    pub fn edit_commands_of_event(menu: &MenuState, event: &Event) -> Result<Vec<EditCommand>, ()> {
        match (event, ctx_tag(&menu.ctx), tree_tag(&menu.tree)) {
            (&Event::Quit { .. }, _, _) => Err(()),
            (&Event::KeyDown(ref kei), ref ctx, ref tree) => match (kei.key.as_str(), ctx, tree) {
                ("Escape", _, _) => Err(()),
                //("Enter", Tag::Root, _) => Ok(vec![EditCommand::Commit]),
                ("Enter", _, _) => Ok(vec![EditCommand::Commit]),
                ("Backspace", _, _) => Ok(vec![EditCommand::Clear]),

                ("Tab", _, Tag::Blank) => Ok(vec![EditCommand::AutoFill]),
                ("ArrowRight", _, Tag::Blank) => Ok(vec![EditCommand::AutoFill]),

                ("ArrowLeft", _, _) => Ok(vec![EditCommand::Ascend]),
                ("ArrowRight", _, _) => Ok(vec![EditCommand::Descend]),

                ("ArrowUp", _, Tag::Variant) => Ok(vec![EditCommand::PrevVariant]),
                ("ArrowDown", _, Tag::Variant) => Ok(vec![EditCommand::NextVariant]),

                ("ArrowUp", Tag::Product, _) => Ok(vec![EditCommand::PrevSibling]),
                ("ArrowDown", Tag::Product, _) => Ok(vec![EditCommand::NextSibling]),

                ("ArrowUp", Tag::Tup, _) => Ok(vec![EditCommand::PrevSibling]),
                ("ArrowDown", Tag::Tup, _) => Ok(vec![EditCommand::NextSibling]),

                (key, ctx, tree) => {
                    warn!(
                        "unrecognized key-tag combo: {:?} on a {:?} context, holding a {:?}",
                        key, ctx, tree
                    );
                    Ok(vec![])
                }
            },
            (ev, _, _) => {
                warn!("unrecognized event: {:?}", ev);
                Ok(vec![])
            }
        }
    }

    use crate::render::{FlowAtts, FrameType, TextAtts};

    pub fn render_elms(menu: &MenuState, r: &mut Render) {
        fn black_fill() -> Fill {
            //Fill::Closed(Color::RGB(0, 0, 0))
            Fill::None
        }

        fn ctx_box_fill() -> Fill {
            Fill::Open(Color::RGB(255, 100, 255), 1)
        }

        fn tree_box_fill() -> Fill {
            Fill::Open(Color::RGB(50, 100, 50), 1)
        }

        fn detailed_tree_box_fill() -> Fill {
            Fill::Open(Color::RGB(100, 255, 100), 1)
        }

        fn active_cursor_fill() -> Fill {
            Fill::Open(Color::RGB(255, 255, 255), 1)
        }

        fn choice_box_fill() -> Fill {
            Fill::Open(Color::RGB(255, 255, 255), 1)
        }

        fn text_zoom() -> usize {
            3
        }
        fn glyph_padding() -> usize {
            1
        }
        fn horz_flow() -> FlowAtts {
            FlowAtts {
                dir: Dir2D::Right,
                intra_pad: 2,
                inter_pad: 2,
            }
        };
        fn vert_flow() -> FlowAtts {
            FlowAtts {
                dir: Dir2D::Down,
                intra_pad: 2,
                inter_pad: 2,
            }
        };

        // eventaually we get these atts from
        //  some environment-determined settings
        fn glyph_flow() -> FlowAtts {
            FlowAtts {
                dir: Dir2D::Right,
                intra_pad: glyph_padding(),
                inter_pad: glyph_padding(),
            }
        }
        fn glyph_dim() -> Dim {
            Dim {
                width: 5,
                height: 5,
            }
        }
        fn kw_atts() -> TextAtts {
            TextAtts {
                zoom: text_zoom(),
                fg_fill: Fill::Closed(Color::RGB(255, 255, 255)),
                bg_fill: Fill::None,
                glyph_dim: glyph_dim(),
                glyph_flow: glyph_flow(),
            }
        };
        fn msg_atts() -> TextAtts {
            TextAtts {
                zoom: 2,
                fg_fill: Fill::Closed(Color::RGB(200, 200, 255)),
                bg_fill: Fill::None,
                glyph_dim: glyph_dim(),
                glyph_flow: glyph_flow(),
            }
        };
        fn typ_lab_atts() -> TextAtts {
            TextAtts {
                zoom: 2,
                fg_fill: Fill::Closed(Color::RGB(200, 255, 255)),
                bg_fill: Fill::None,
                glyph_dim: glyph_dim(),
                glyph_flow: glyph_flow(),
            }
        };
        fn typ_sym_atts() -> TextAtts {
            TextAtts {
                zoom: 2,
                fg_fill: Fill::Closed(Color::RGB(180, 200, 200)),
                bg_fill: Fill::None,
                glyph_dim: glyph_dim(),
                glyph_flow: glyph_flow(),
            }
        };
        fn typ_vflow() -> FlowAtts {
            FlowAtts {
                dir: Dir2D::Down,
                intra_pad: 1,
                inter_pad: 1,
            }
        };
        fn typ_hflow() -> FlowAtts {
            FlowAtts {
                dir: Dir2D::Right,
                intra_pad: 1,
                inter_pad: 1,
            }
        };
        fn text_atts() -> TextAtts {
            TextAtts {
                zoom: text_zoom(),
                fg_fill: Fill::Closed(Color::RGB(200, 200, 200)),
                bg_fill: Fill::None,
                glyph_dim: glyph_dim(),
                glyph_flow: glyph_flow(),
            }
        };
        fn blank_atts() -> TextAtts {
            TextAtts {
                zoom: text_zoom(),
                fg_fill: Fill::Closed(Color::RGB(255, 200, 200)),
                bg_fill: Fill::Closed(Color::RGB(100, 0, 0)),
                glyph_dim: glyph_dim(),
                glyph_flow: glyph_flow(),
            }
        };
        fn pad_atts() -> TextAtts {
            TextAtts {
                zoom: 1,
                fg_fill: Fill::None,
                bg_fill: Fill::None,
                glyph_dim: glyph_dim(),
                glyph_flow: glyph_flow(),
            }
        };
        fn cursor_atts() -> TextAtts {
            TextAtts {
                zoom: text_zoom(),
                fg_fill: Fill::Closed(Color::RGB(255, 255, 255)),
                bg_fill: Fill::None,
                glyph_dim: glyph_dim(),
                glyph_flow: glyph_flow(),
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
                r.str(">", &cursor_atts());
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

        fn begin_flow(r: &mut Render, f: &FlowAtts) {
            r.begin(&Name::Void, FrameType::Flow(f.clone()))
        }

        fn render_ctx(ctx: &MenuCtx, r_out: &mut Render, r_tree: Render) {
            let mut next_ctx = None;
            let mut r = Render::new();

            r.begin(&Name::Void, FrameType::Flow(horz_flow()));
            r.fill(ctx_box_fill());

            match ctx {
                &MenuCtx::Root(ref _t) => {
                    drop(r);
                    if let Some(_) = next_ctx {
                        unreachable!()
                    };
                    r_out.begin(&Name::Void, FrameType::Flow(vert_flow()));
                    r_out.nest(&Name::Void, r_tree);
                    r_out.end();
                    return;
                }
                &MenuCtx::Tup(ref sel) => {
                    next_ctx = Some(sel.ctx.clone());
                    r.begin(&Name::Void, FrameType::Flow(vert_flow()));
                    for (t, _ty) in sel.before.iter() {
                        begin_item(&mut r);
                        //render_product_label(l, &mut r);
                        render_tree(t, false, &ctx_box_fill(), &mut r);
                        r.end();
                    }
                    {
                        begin_item(&mut r);
                        //render_product_label(&sel.label, &mut r);
                        r.nest(&Name::Void, r_tree);
                        r.end();
                    }
                    for (t, _ty) in sel.after.iter() {
                        begin_item(&mut r);
                        //render_product_label(&l, &mut r);
                        render_tree(t, false, &ctx_box_fill(), &mut r);
                        r.end();
                    }
                    r.end();
                }
                &MenuCtx::Product(ref sel) => {
                    next_ctx = Some(sel.ctx.clone());
                    r.begin(&Name::Void, FrameType::Flow(vert_flow()));
                    for (l, t, _ty) in sel.before.iter() {
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
                    for (l, t, _ty) in sel.after.iter() {
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
                &MenuCtx::Option(_flag, ref _body) => unimplemented!(),
                &MenuCtx::Vec(ref _ch) => unimplemented!(),
            };
            r.end();
            // continue rendering the rest of the context, in whatever flow we are using for that purpose.
            if let Some(ctx) = next_ctx {
                render_ctx(&ctx, r_out, r)
            } else {
                //info!("context end: root.");
            };
        };

        use super::{MenuType, PrimType};

        fn render_type(
            typ: &MenuType,
            text: &TextAtts,
            text2: &TextAtts,
            vflow: &FlowAtts,
            hflow: &FlowAtts,
            r: &mut Render,
        ) {
            let mut first = true;
            match typ {
                MenuType::Prim(PrimType::Unit) => r.str("()", text),
                MenuType::Prim(PrimType::Null) => r.str("null", text),
                MenuType::Prim(PrimType::Nat) => r.str("nat", text),
                MenuType::Prim(PrimType::Int) => r.str("int", text),
                MenuType::Prim(PrimType::Text) => r.str("text", text),
                MenuType::Prim(PrimType::Bool) => r.str("bool", text),
                MenuType::Variant(fields) => {
                    begin_flow(r, vflow);
                    if fields.len() > 0 {
                        for (l, t) in fields.iter() {
                            if first {
                                first = false;
                                begin_flow(r, hflow);
                                r.str("{ ", text2);
                            } else {
                                r.end();
                                begin_flow(r, hflow);
                                r.str("; ", text2);
                            };
                            r.str("#", text2);
                            r.str(&format!("{}", l), text);
                            r.str(": ", text2);
                            render_type(t, text, text2, vflow, hflow, r);
                        }
                        r.str(" }", text2);
                        r.end();
                    } else {
                        unimplemented!()
                    }
                    r.end()
                }
                MenuType::Product(fields) => {
                    begin_flow(r, vflow);
                    if fields.len() > 0 {
                        for (l, t) in fields.iter() {
                            if first {
                                first = false;
                                begin_flow(r, hflow);
                                r.str("{ ", text2);
                            } else {
                                r.end();
                                begin_flow(r, hflow);
                                r.str("; ", text2);
                            };
                            r.str(&format!("{}", l), text);
                            r.str(":", text2);
                            render_type(t, text, text2, vflow, hflow, r);
                        }
                        r.str(" }", text2);
                        r.end();
                    } else {
                        r.str("{ }", text2);
                    }
                    r.end()
                }
                MenuType::Var(n) => r.name(n, text),
                MenuType::Option(t) => {
                    begin_flow(r, hflow);
                    r.str("?", text2);
                    render_type(t, text, text2, vflow, hflow, r);
                    r.end()
                }
                MenuType::Vec(t) => {
                    begin_flow(r, hflow);
                    r.str("[", text2);
                    render_type(t, text, text2, vflow, hflow, r);
                    r.str("]", text2);
                    r.end()
                }
                MenuType::Tup(typs) => {
                    begin_flow(r, hflow);
                    r.str("(", text2);
                    let mut not_first = false;
                    for t in typs.iter() {
                        if not_first {
                            r.str(", ", text2);
                        }
                        render_type(t, text, text2, vflow, hflow, r);
                        not_first = true;
                    }
                    r.str(")", text2);
                    r.end()
                }
                MenuType::Func(ft) => {
                    begin_flow(r, hflow);
                    r.str("(", text2);
                    let mut not_first = false;
                    for t in ft.args.iter() {
                        if not_first {
                            r.str(", ", text2);
                        }
                        render_type(t, text, text2, vflow, hflow, r);
                        not_first = true;
                    }
                    r.str(")", text2);
                    r.str("→", text2);
                    r.str("(", text2);
                    let mut not_first = false;
                    for t in ft.rets.iter() {
                        if not_first {
                            r.str(", ", text2);
                        }
                        render_type(t, text, text2, vflow, hflow, r);
                        not_first = true;
                    }
                    r.str(")", text2);
                    r.end()
                }
            }
        }

        fn render_tree(tree: &MenuTree, show_detailed: bool, box_fill: &Fill, r: &mut Render) {
            r.begin(&Name::Void, FrameType::Flow(horz_flow()));
            r.fill(box_fill.clone());
            let box_fill = &tree_box_fill();
            match tree {
                &MenuTree::Product(ref fields) => {
                    r.begin(&Name::Void, FrameType::Flow(vert_flow()));
                    for (l, t, _ty) in fields.iter() {
                        begin_item(r);
                        render_product_label(l, r);
                        render_tree(t, false, box_fill, r);
                        r.end()
                    }
                    r.end()
                }
                &MenuTree::Variant(ref ch) => {
                    r.begin(&Name::Void, FrameType::Flow(vert_flow()));
                    if show_detailed {
                        begin_item(r);
                        r.text(&format!("choice:"), &msg_atts());
                        if let Some(_) = ch.choice {
                            // nothing
                        } else {
                            r.text(&format!("___"), &blank_atts());
                            r.text(&format!(" Please, choose one below (Up/Down)"), &msg_atts());
                        }
                        r.end();
                        r.begin(&Name::Void, FrameType::Flow(vert_flow()));
                        r.fill(choice_box_fill());
                        for (l, t, _ty) in ch.before.iter() {
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
                        } else {
                            if false {
                                begin_item(r);
                                r.text(&format!(">"), &cursor_atts());
                                r.text(&format!("___"), &blank_atts());
                                r.text(
                                    &format!(" Please, choose one below (Up/Down)"),
                                    &msg_atts(),
                                );
                                r.end();
                            }
                        };
                        for (l, t, _ty) in ch.after.iter() {
                            begin_item(r);
                            render_variant_label(false, l, r);
                            render_tree(t, false, box_fill, r);
                            r.end()
                        }
                        r.end();
                    } else {
                        if let Some((ref label, ref tree, _)) = ch.choice {
                            begin_item(r);
                            render_choice_label(&label, r);
                            render_tree(tree, false, box_fill, r);
                            r.end();
                        } else {
                            r.text(&format!("___"), &blank_atts());
                        }
                    }
                    r.end();
                }
                &MenuTree::Option(flag, ref tree, ref _typ) => {
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
                &MenuTree::Blank(ref _typ) => r.text(&format!("___"), &blank_atts()),
                &MenuTree::Nat(n) => r.text(&format!("{}", n), &text_atts()),
                &MenuTree::Int(i) => r.text(&format!("{}", i), &text_atts()),
                &MenuTree::Bool(b) => r.text(&format!("{}", b), &text_atts()),
                &MenuTree::Text(ref t) => r.text(&format!("{:?}", t), &text_atts()),
                &MenuTree::Unit => r.str("()", &text_atts()),
                &MenuTree::Null => r.str("null", &text_atts()),
            };
            r.end();
        };
        r.begin(&Name::Void, FrameType::Flow(vert_flow()));
        if true {
            if false {
                r.str("hello world!", &text_atts());
                r.str(" please, enter a value to submit:", &msg_atts());
                r.str(" (Auto-fill and navigate with arrow keys)", &msg_atts());
            }
            let r_tree = {
                let mut r_tree = Render::new();
                r_tree.begin(&Name::Void, FrameType::Flow(vert_flow()));
                r_tree.fill(active_cursor_fill());
                render_tree(&menu.tree, true, &detailed_tree_box_fill(), &mut r_tree);
                render_type(
                    &menu.tree_typ,
                    &typ_lab_atts(),
                    &typ_sym_atts(),
                    &typ_vflow(),
                    &typ_hflow(),
                    &mut r_tree,
                );
                r_tree.end();
                r_tree
            };
            render_ctx(&menu.ctx, r, r_tree);
        }
        r.end();
    }
}

use std::fmt;

impl fmt::Display for MenuTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        match self {
            MenuTree::Product(ref fields) => {
                write!(f, "record {{")?;
                for (l, t, _) in fields.iter() {
                    if !first {
                        write!(f, "; ")?;
                    };
                    first = false;
                    write!(f, "{}={}", l, t)?;
                }
                write!(f, "}}")
            }
            MenuTree::Variant(ref choice) => {
                write!(f, "variant {{")?;
                if let Some((ref l, ref t, _)) = choice.choice {
                    write!(f, "{}={}", l, t)?;
                } else {
                    // nothing
                }
                write!(f, "}}")
            }
            MenuTree::Option(ref b, ref t, _) => {
                if *b {
                    write!(f, "opt {}", t)
                } else {
                    write!(f, "opt null")
                }
            }
            MenuTree::Vec(ref ts, _) => {
                write!(f, "vec{{")?;
                for t in ts.iter() {
                    if !first {
                        write!(f, "; ")?;
                    };
                    first = false;
                    write!(f, "{}", t)?;
                }
                write!(f, "}}")
            }
            MenuTree::Tup(ref ts) => {
                write!(f, "(")?;
                for (t, _) in ts.iter() {
                    if !first {
                        write!(f, ", ")?;
                    };
                    first = false;
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            MenuTree::Blank(_) => write!(f, "variant {{BLANK=()}}"),
            MenuTree::Nat(n) => write!(f, "{}", n),
            MenuTree::Int(i) => write!(f, "{}", i),
            MenuTree::Text(t) => write!(f, "{:?}", t),
            MenuTree::Bool(b) => write!(f, "{}", b),
            MenuTree::Unit => write!(f, "()"),
            MenuTree::Null => write!(f, "null"),
        }
    }
}

// move elsewhere
impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Name::Void => write!(f, "<void>"),
            Name::Atom(Atom::Bool(b)) => write!(f, "{}", b),
            Name::Atom(Atom::Usize(u)) => write!(f, "{}", u),
            Name::Atom(Atom::String(s)) => write!(f, "{}", s),
            Name::Merkle(_m) => unimplemented!(),
            Name::TaggedTuple(n, ns) => {
                write!(f, "{}", n)?;
                if ns.len() > 0 {
                    write!(f, "(")?;
                    for n in ns.iter() {
                        write!(f, "{}", n)?;
                    }
                    write!(f, ")")
                } else {
                    Ok(())
                }
            }
        }
    }
}

impl fmt::Display for MenuType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        match self {
            MenuType::Var(ref n) => write!(f, "{:?}", n),
            MenuType::Prim(PrimType::Unit) => write!(f, "()"),
            MenuType::Prim(PrimType::Null) => write!(f, "null"),
            MenuType::Prim(PrimType::Nat) => write!(f, "nat"),
            MenuType::Prim(PrimType::Int) => write!(f, "int"),
            MenuType::Prim(PrimType::Text) => write!(f, "text"),
            MenuType::Prim(PrimType::Bool) => write!(f, "bool"),
            MenuType::Variant(fields) => {
                write!(f, "{{")?;
                for (l, t) in fields.iter() {
                    if !first {
                        write!(f, "; ")?;
                    };
                    write!(f, "#{}: {}", l, t)?;
                    first = false;
                }
                write!(f, "}}")
            }
            MenuType::Product(fields) => {
                write!(f, "{{")?;
                for (l, t) in fields.iter() {
                    if !first {
                        write!(f, "; ")?;
                    };
                    write!(f, "{}: {}", l, t)?;
                    first = false;
                }
                write!(f, "}}")
            }
            MenuType::Option(t) => write!(f, "?{}", t),
            MenuType::Vec(t) => write!(f, "[{}]", t),
            MenuType::Tup(fields) => {
                write!(f, "(")?;
                for t in fields.iter() {
                    if !first {
                        write!(f, ", ")?;
                    };
                    write!(f, "{}", t)?;
                    first = false;
                }
                write!(f, ")")
            }
            MenuType::Func(ft) => {
                write!(f, "(")?;
                for t in ft.args.iter() {
                    if !first {
                        write!(f, ", ")?;
                    };
                    write!(f, "{}", t)?;
                    first = false;
                }
                write!(f, ") -> (");
                for t in ft.rets.iter() {
                    if !first {
                        write!(f, ", ")?;
                    };
                    write!(f, "{}", t)?;
                    first = false;
                }
                write!(f, ")")
            }
        }
    }
}
