extern crate serde_idl;

use menu;
use menu::MenuType;
use types::lang::{Atom, Name};

use serde_idl::grammar::IDLProgParser;
use serde_idl::lexer::Lexer;
use serde_idl::types::{to_pretty, Dec, IDLProg, IDLType, Label};

use std::collections::HashMap;
pub type Env = HashMap<String, MenuType>;

pub fn parse_idl(input: &str) -> IDLProg {
    let lexer = Lexer::new(input);
    IDLProgParser::new().parse(lexer).unwrap()
}

fn name_of_idllabel(l: &Label) -> Name {
    match l {
        Label::Id(n) => Name::Atom(Atom::Usize(*n as usize)),
        Label::Named(n) => Name::Atom(Atom::String(n.clone())),
        Label::Unnamed(_) => Name::Void,
    }
}

fn menutype_of_idltype(env: &Env, t: &IDLType) -> MenuType {
    match t {
        IDLType::RecordT(fields) => {
            let mut out = vec![];
            for field in fields.iter() {
                let n = name_of_idllabel(&field.label);
                let t = menutype_of_idltype(env, &field.typ);
                out.push((n, t))
            }
            MenuType::Product(out)
        }
        IDLType::VariantT(fields) => {
            let mut out = vec![];
            for field in fields.iter() {
                let n = name_of_idllabel(&field.label);
                let t = menutype_of_idltype(env, &field.typ);
                out.push((n, t))
            }
            MenuType::Variant(out)
        }
        IDLType::OptT(t) => {
            let mt = menutype_of_idltype(env, t);
            MenuType::Option(Box::new(mt))
        }
        IDLType::VarT(v) => menutype_resolve_var(env, v, 0),
        IDLType::VecT(t) => {
            let t = menutype_of_idltype(env, &*t);
            MenuType::Vec(Box::new(t))
        }
        IDLType::FuncT(ft) => {
            let args = ft
                .args
                .iter()
                .map(|t| menutype_of_idltype(env, t))
                .collect();
            let rets = ft
                .rets
                .iter()
                .map(|t| menutype_of_idltype(env, t))
                .collect();
            MenuType::Func(menu::FuncType { args, rets })
        }
        IDLType::PrimT(Nat) => MenuType::Prim(menu::PrimType::Nat),
        IDLType::PrimT(Text) => MenuType::Prim(menu::PrimType::Text),
        IDLType::PrimT(Bool) => MenuType::Prim(menu::PrimType::Bool),
        _ => unimplemented!("{:?}", t),
    }
}

pub fn menutype_resolve_var(env: &Env, v: &String, depth: usize) -> MenuType {
    if depth > 100 {
        MenuType::Var(Name::Atom(Atom::String(v.clone())))
    } else {
        match env.get(v) {
            None => MenuType::Var(Name::Atom(Atom::String(v.clone()))),
            Some(MenuType::Var(Name::Atom(Atom::String(v)))) => {
                menutype_resolve_var(env, v, depth + 1)
            }
            Some(MenuType::Var(_)) => unreachable!(),
            Some(t) => t.clone(),
        }
    }
}

pub fn menutype_of_idlprog_service(p: &IDLProg) -> menu::MenuType {
    let mut emp = HashMap::new();
    let mut env = HashMap::new();
    for dec in p.decs.iter() {
        match dec {
            Dec::TypD(ref b) => {
                let t = menutype_of_idltype(&emp, &b.typ);
                //print!("{:?}", b);
                drop(env.insert(b.id.clone(), t))
            }
            _ => unimplemented!(),
        }
    }
    match p.actor {
        Some(IDLType::ServT(ref methods)) => {
            let mut choices = vec![];
            for method in methods.iter() {
                //eprint!("method {:?} : {:?}", method.id, method.typ);
                let i = method.id.clone();
                let t = match method.typ {
                    IDLType::FuncT(ref ft) => {
                        let arg_types: Vec<MenuType> = ft
                            .args
                            .iter()
                            .map(|a| menutype_of_idltype(&env, a))
                            .collect();
                        let mut fields = vec![];
                        for i in 0..arg_types.len() {
                            fields
                                .push((Name::Atom(Atom::Usize(fields.len())), arg_types[i].clone()))
                        }
                        MenuType::Product(fields)
                    }
                    _ => unreachable!(),
                };
                choices.push((Name::Atom(Atom::String(i)), t));
            }
            MenuType::Variant(choices)
        }
        _ => panic!("expected a service type"),
    }
}

use types::lang::{Command, Editor, Frame, State};
pub fn init_of_idlprog_ast(p: &IDLProg) -> Result<State, String> {
    use eval;
    let mt: menu::MenuType = menutype_of_idlprog_service(p);
    let mut st = State {
        stack: vec![],
        frame: Frame::from_editor(Editor::Menu(Box::new(menu::Editor {
            state: None,
            history: vec![],
        }))),
    };
    let cmd = Command::Menu(menu::Command::Init(menu::InitCommand::Default(
        menu::MenuTree::Blank(mt.clone()),
        mt,
    )));
    eval::command_eval(&mut st, &cmd)?;
    Ok(st)
}

#[test]
fn doit() {
    let prog = r#"
service server : {
  f : (a: nat, b:nat) -> () oneway;
  g : (a: text, b:nat) -> () oneway;
  h : (a: nat, b:record { nat; nat; record { nat; 0x2a:nat; nat8; }; 42:nat; 40:nat; variant{ A; 0x2a; B; C }; }) -> () oneway;
}
    "#;
    let ast = parse_idl(&prog);
    let menu = menutype_of_idlprog_service(&ast);
    drop(menu);
}
