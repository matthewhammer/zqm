extern crate serde_idl;

use menu;
use menu::MenuType;
use types::lang::{Atom, Name};

use serde_idl::grammar::IDLProgParser;
use serde_idl::lexer::Lexer;
use serde_idl::types::{to_pretty, IDLProg, IDLType, Label};

pub fn parse_idl(input: &str) -> IDLProg {
    let lexer = Lexer::new(input);
    IDLProgParser::new().parse(lexer).unwrap()
}

pub fn name_of_idllabel(l: &Label) -> Name {
    match l {
        Label::Id(n) => Name::Atom(Atom::Usize(*n as usize)),
        Label::Named(n) => Name::Atom(Atom::String(n.clone())),
        Label::Unnamed(_) => Name::Void,
    }
}

pub fn menutype_of_idltype(t: &IDLType) -> MenuType {
    match t {
        IDLType::RecordT(fields) => {
            let mut out = vec![];
            for field in fields.iter() {
                let n = name_of_idllabel(&field.label);
                let t = menutype_of_idltype(&field.typ);
                out.push((n, t))
            }
            MenuType::Product(out)
        }
        IDLType::VariantT(fields) => {
            let mut out = vec![];
            for field in fields.iter() {
                let n = name_of_idllabel(&field.label);
                let t = menutype_of_idltype(&field.typ);
                out.push((n, t))
            }
            MenuType::Variant(out)
        }
        IDLType::PrimT(Nat) => MenuType::Prim(menu::PrimType::Nat),
        IDLType::PrimT(Text) => MenuType::Prim(menu::PrimType::Text),
        IDLType::PrimT(Bool) => MenuType::Prim(menu::PrimType::Bool),
        _ => unimplemented!("{:?}", t),
    }
}

pub fn menutype_of_idlprog_service(p: &IDLProg) -> menu::MenuType {
    match p.actor {
        Some(IDLType::ServT(ref methods)) => {
            let mut choices = vec![];
            for method in methods.iter() {
                //eprint!("method {:?} : {:?}", method.id, method.typ);
                let i = method.id.clone();
                let t = match method.typ {
                    IDLType::FuncT(ref ft) => {
                        let arg_types: Vec<MenuType> =
                            ft.args.iter().map(|a| menutype_of_idltype(a)).collect();
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
