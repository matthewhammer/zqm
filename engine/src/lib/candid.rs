use serde::{Deserialize, Serialize};
extern crate serde_idl;

use menu;
use menu::MenuType;
use types::{
    lang::{Atom, Name},
    render,
};

use ic_http_agent::{Agent, AgentConfig, Blob, CanisterId};
use serde_idl::grammar::IDLProgParser;
use serde_idl::lexer::Lexer;
use serde_idl::{
    types::{Dec, IDLProg, IDLType, Label, PrimType},
    value::IDLArgs,
    value::IDLField,
    value::IDLValue,
};

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
        IDLType::PrimT(PrimType::Nat) => MenuType::Prim(menu::PrimType::Nat),
        IDLType::PrimT(PrimType::Int) => MenuType::Prim(menu::PrimType::Int),
        IDLType::PrimT(PrimType::Nat8) => MenuType::Prim(menu::PrimType::Nat),
        IDLType::PrimT(PrimType::Text) => MenuType::Prim(menu::PrimType::Text),
        IDLType::PrimT(PrimType::Bool) => MenuType::Prim(menu::PrimType::Bool),
        IDLType::PrimT(PrimType::Null) => MenuType::Prim(menu::PrimType::Null),
        _ => unimplemented!("{:?}", t),
    }
}

pub fn string_of_field_id(id: u32) -> Option<String> {
    match id {
        24860 => Some("ok".to_string()),
        1269255460 => Some("rect".to_string()),
        4996424 => Some("dim".to_string()),
        5594516 => Some("pos".to_string()),
        38537191 => Some("height".to_string()),
        3395466758 => Some("width".to_string()),
        120 => Some("x".to_string()),
        121 => Some("y".to_string()),
        240232876 => Some("closed".to_string()),
        _ => None,
    }
}

pub fn get_nat(n: &IDLValue) -> Option<usize> {
    match n {
        IDLValue::Nat(n) => Some(*n as usize),
        _ => None,
    }
}
pub fn get_pos(pos: &IDLValue) -> Option<render::Pos> {
    match pos {
        IDLValue::Record(fields) => match (get_nat(&fields[0].val), get_nat(&fields[1].val)) {
            (Some(x), Some(y)) => {
                let x = x as isize;
                let y = y as isize;
                Some(render::Pos { x, y })
            }
            _ => None,
        },
        _ => None,
    }
}

pub fn get_dim(dim: &IDLValue) -> Option<render::Dim> {
    match dim {
        IDLValue::Record(fields) => match (get_nat(&fields[0].val), get_nat(&fields[1].val)) {
            (Some(width), Some(height)) => Some(render::Dim { width, height }),
            _ => None,
        },
        _ => None,
    }
}

pub fn get_rect(rect: &IDLValue) -> Option<render::Rect> {
    match rect {
        IDLValue::Record(fields) => {
            if fields.len() != 2 {
                return None;
            };
            match (get_pos(&fields[1].val), get_dim(&fields[0].val)) {
                (Some(pos), Some(dim)) => Some(render::Rect { pos, dim }),
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn get_color(color: &IDLValue) -> Option<render::Color> {
    match color {
        IDLValue::Record(fields) => {
            if fields.len() != 3 {
                return None;
            };
            match (
                get_nat(&fields[0].val),
                get_nat(&fields[1].val),
                get_nat(&fields[2].val),
            ) {
                (Some(r), Some(g), Some(b)) => Some(render::Color::RGB(r, g, b)),
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn get_fill_(fill: &IDLField) -> Option<render::Fill> {
    match fill.id {
        // #closed : Color
        240232876 => match get_color(&fill.val) {
            Some(c) => Some(Fill::Closed(c)),
            None => None,
        },
        _ => None,
    }
}

pub fn get_fill(fill: &IDLValue) -> Option<render::Fill> {
    match fill {
        IDLValue::Variant(v) => get_fill_(&*v),
        _ => None,
    }
}

pub fn render_rect_fill(elm: &IDLValue) -> Option<render::Elm> {
    match elm {
        IDLValue::Record(fields) => {
            if fields.len() != 2 {
                return None;
            };
            match (get_rect(&fields[0].val), get_fill(&fields[1].val)) {
                (Some(r), Some(f)) => Some(render::Elm::Rect(r, f)),
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn render_elm(elm: &IDLField) -> Option<render::Elm> {
    info!("render_elm");
    match elm.id {
        // #rect : (Rect, Fill)
        1269255460 => render_rect_fill(&elm.val),
        _ => {
            info!("warning: recognized element {:?}", elm);
            None
        }
    }
}

pub fn render_ok(elms: &IDLValue) -> Option<render::Elms> {
    info!("render_ok");
    match elms {
        IDLValue::Vec(vals) => {
            let mut out: render::Elms = vec![];
            for v in vals {
                match v {
                    IDLValue::Variant(elm) => match render_elm(elm) {
                        None => {
                            warn!("unrecognized element {:?}", v);
                            return None;
                        }
                        Some(elm) => out.push(elm),
                    },
                    _ => {
                        warn!("unrecognized element {:?}", v);
                        return None;
                    }
                }
            }
            Some(out)
        }
        _ => {
            warn!("expected Vec, not {:?}", elms);
            None
        }
    }
}

pub fn render_err(ret: &IDLValue) -> Option<render::Elms> {
    None
}

pub fn render_result(ret: &IDLValue) -> Option<render::Elms> {
    match ret {
        IDLValue::Variant(f) => {
            match f.id {
                24860 => render_ok(&f.val),
                666 => render_err(&f.val), // to do
                _ => {
                    warn!("unexpected field {:?}", f);
                    None
                }
            }
        }
        IDLValue::Record(fs) => None,
        _ => None,
    }
}

pub fn render_of_rets(rets: &Vec<IDLValue>) -> Option<render::Elms> {
    trace!("{:?}", rets);
    let mut out: render::Elms = vec![];
    for res in rets.iter() {
        match render_result(res) {
            None => return None,
            Some(mut x) => out.append(&mut x),
        }
    }
    Some(out)
}

pub fn idlargs_of_menutree(mt: &menu::MenuTree) -> String {
    format!("{}", mt)
}

pub fn blob_of_menutree(mt: &menu::MenuTree) -> Blob {
    let args: String = idlargs_of_menutree(mt);
    let args: IDLArgs = args.parse().unwrap();
    Blob(args.to_bytes().unwrap())
}

pub fn menutype_resolve_var(env: &Env, v: &String, depth: usize) -> MenuType {
    // to do -- what is the real threshold in the def again?
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

pub fn tuple_of_product(t: &menu::MenuTree) -> menu::MenuTree {
    // to do
    t.clone()
}

pub fn menutype_of_idlprog(p: &IDLProg) -> menu::MenuType {
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
                        if ft.args.len() > 0 {
                            let arg_types: Vec<MenuType> = ft
                                .args
                                .iter()
                                .map(|a| menutype_of_idltype(&env, a))
                                .collect();
                            let mut fields = vec![];
                            for i in 0..arg_types.len() {
                                fields.push(arg_types[i].clone())
                            }
                            MenuType::Tup(fields)
                        } else {
                            MenuType::Prim(menu::PrimType::Unit)
                        }
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

pub fn agent(url: &str) -> Result<Agent, ic_http_agent::AgentError> {
    Agent::new(AgentConfig {
        url: format!("http://{}", url).as_str(),
        ..AgentConfig::default()
    })
}

use std::time::{Duration, SystemTime};

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Call {
    pub method: String,
    pub args: menu::MenuTree,
    pub args_idl: String,
    pub rets_idl: Result<String, String>,
    pub rets_render: Option<render::Elms>,
    pub timestamp: SystemTime,
    pub duration: Duration,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Repl {
    pub config: Config,
    pub history: Vec<Call>,
    // todo: log of results, parsed into MenuTree's according to the MenuType of the result type
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Config {
    //idl_prog: IDLProg,
    pub replica_url: String,
    pub canister_id: String,
    pub menu_type: MenuType,
}

use types::lang::{Command, Editor, Frame, State};
pub fn init(url: &str, cid_text: &str, p: &IDLProg) -> Result<State, String> {
    use eval;
    let cid: CanisterId = CanisterId::from_text(cid_text).unwrap();
    assert_eq!(cid.to_text(), cid_text);
    let mt: menu::MenuType = menutype_of_idlprog(p);
    let mut st = State {
        stack: vec![Frame::from_editor(Editor::CandidRepl(Box::new(Repl {
            history: vec![],
            config: Config {
                //idl_prog: p.clone(),
                menu_type: mt.clone(),
                replica_url: url.to_string(),
                canister_id: cid.to_string(),
            },
        })))],
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

use crate::render::{FlowAtts, FrameType, Render, TextAtts};
use types::{
    lang::Dir2D,
    render::{Color, Dim, Elms, Fill},
};

pub fn render_elms(repl: &Repl, r: &mut Render) {
    fn text_zoom() -> usize {
        2
    }
    fn horz_flow() -> FlowAtts {
        FlowAtts {
            dir: Dir2D::Right,
            intra_pad: 2,
            inter_pad: 2,
        }
    }
    fn vert_flow() -> FlowAtts {
        FlowAtts {
            dir: Dir2D::Down,
            intra_pad: 2,
            inter_pad: 2,
        }
    }
    fn glyph_padding() -> usize {
        1
    }
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
            fg_fill: Fill::Closed(Color::RGB(255, 230, 255)),
            bg_fill: Fill::None,
            glyph_dim: glyph_dim(),
            glyph_flow: glyph_flow(),
        }
    }
    fn dim_atts() -> TextAtts {
        TextAtts {
            zoom: text_zoom(),
            fg_fill: Fill::Closed(Color::RGB(150, 100, 150)),
            bg_fill: Fill::None,
            glyph_dim: glyph_dim(),
            glyph_flow: glyph_flow(),
        }
    }
    fn dim2_atts() -> TextAtts {
        TextAtts {
            zoom: text_zoom(),
            fg_fill: Fill::Closed(Color::RGB(200, 180, 200)),
            bg_fill: Fill::None,
            glyph_dim: glyph_dim(),
            glyph_flow: glyph_flow(),
        }
    }
    fn data_atts() -> TextAtts {
        TextAtts {
            zoom: text_zoom(),
            fg_fill: Fill::Closed(Color::RGB(230, 230, 230)),
            bg_fill: Fill::None,
            glyph_dim: glyph_dim(),
            glyph_flow: glyph_flow(),
        }
    };
    fn err_atts() -> TextAtts {
        TextAtts {
            zoom: text_zoom(),
            fg_fill: Fill::Closed(Color::RGB(255, 100, 100)),
            bg_fill: Fill::None,
            glyph_dim: glyph_dim(),
            glyph_flow: glyph_flow(),
        }
    };
    fn msg_atts() -> TextAtts {
        TextAtts {
            zoom: text_zoom(),
            fg_fill: Fill::Closed(Color::RGB(200, 200, 255)),
            bg_fill: Fill::None,
            glyph_dim: glyph_dim(),
            glyph_flow: glyph_flow(),
        }
    };
    fn box_fill() -> Fill {
        Fill::Open(Color::RGB(50, 100, 50), 1)
    };

    if repl.history.len() > 0 {
        r.begin(&Name::Void, FrameType::Flow(vert_flow()));
        r.str("Message log:", &msg_atts());
        r.begin(&Name::Void, FrameType::Flow(vert_flow()));
        r.fill(box_fill());
        for call in repl.history.iter() {
            r.begin(&Name::Void, FrameType::Flow(vert_flow()));
            r.begin(&Name::Void, FrameType::Flow(horz_flow()));
            r.str(&call.method, &msg_atts());
            r.text(&call.args_idl, &data_atts());
            r.end();
            match call.rets_idl {
                Ok(ref rets_idl) => {
                    r.begin(&Name::Void, FrameType::Flow(horz_flow()));
                    r.text(&format!(" {:?}", &call.duration), &dim_atts());
                    r.str("━━►", &dim2_atts());
                    // to do -- if len of text overflows, then wrap it
                    match call.rets_render {
                        None => {
                            r.text(rets_idl, &data_atts());
                        }
                        Some(ref elms) => r.add(elms.clone()),
                    };
                    r.end()
                }
                Err(ref msg) => {
                    r.begin(&Name::Void, FrameType::Flow(vert_flow()));
                    r.begin(&Name::Void, FrameType::Flow(horz_flow()));
                    r.text(&format!(" {:?}", &call.duration), &dim_atts());
                    r.str("━━► ", &err_atts());
                    r.end();
                    r.begin(&Name::Void, FrameType::Flow(horz_flow()));
                    r.str(&" ", &err_atts());
                    // to do -- if len of text overflows, then wrap it
                    r.text(msg, &err_atts());
                    r.end();
                    r.end();
                }
            }
            r.end()
        }
        r.end();
        r.end()
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
