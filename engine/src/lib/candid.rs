use serde::{Deserialize, Serialize};
extern crate serde_idl;

use menu;
use menu::MenuType;
use render;
use types::lang::{Atom, Name};

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
        IDLType::ServT(_) => {
            // To do (!):
            MenuType::Prim(menu::PrimType::Null)
        }
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

pub fn get_nat(v: &IDLValue) -> Option<usize> {
    match v {
        IDLValue::Nat(n) => Some(*n as usize),
        _ => {
            error!("expected nat: {:?}", v);
            None
        }
    }
}

pub fn get_text(v: &IDLValue) -> Option<String> {
    match v {
        IDLValue::Text(t) => Some(t.clone()),
        _ => {
            error!("expected text: {:?}", v);
            None
        }
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
        IDLValue::Record(fields) => match (get_nat(&fields[1].val), get_nat(&fields[0].val)) {
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

pub fn get_color_nat(color_nat: &IDLValue) -> Option<(render::Color, usize)> {
    match color_nat {
        IDLValue::Record(fields) => {
            if fields.len() != 2 {
                return None;
            };
            match (get_color(&fields[0].val), get_nat(&fields[1].val)) {
                (Some(c), Some(n)) => Some((c, n)),
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn get_fill_(f: &IDLField) -> Option<render::Fill> {
    match f.id {
        // #closed : Color
        240232876 => match get_color(&f.val) {
            Some(c) => Some(Fill::Closed(c)),
            None => None,
        },
        // #open : (Color, Nat)
        1236534218 => match get_color_nat(&f.val) {
            Some((c, n)) => Some(Fill::Open(c, n)),
            None => None,
        },
        // #none
        1225396920 => Some(Fill::None),
        _ => {
            error!("unrecognized fill tag: {:?}", f);
            None
        }
    }
}

pub fn get_fill(v: &IDLValue) -> Option<render::Fill> {
    match v {
        IDLValue::Variant(v) => get_fill_(&*v),
        _ => {
            error!("unrecognized fill: {:?}", v);
            None
        }
    }
}

pub fn get_dir2d_(f: &IDLField) -> Option<Dir2D> {
    match f.id {
        3915647964 => Some(Dir2D::Right),
        _ => {
            error!("unrecognized dir2d tag: {:?}", f.id);
            None
        }
    }
}

pub fn get_dir2d(v: &IDLValue) -> Option<Dir2D> {
    match v {
        IDLValue::Variant(v) => get_dir2d_(&*v),
        _ => {
            error!("unrecognized dir2d: {:?}", v);
            None
        }
    }
}

pub fn get_flow_atts(v: &IDLValue) -> Option<render::FlowAtts> {
    match v {
        IDLValue::Record(fields) => {
            if fields.len() != 3 {
                return None;
            };
            match (
                get_dir2d(&fields[0].val),
                get_nat(&fields[1].val),
                get_nat(&fields[2].val),
            ) {
                (Some(dir), Some(intra_pad), Some(inter_pad)) => Some(FlowAtts {
                    dir,
                    intra_pad,
                    inter_pad,
                }),
                _ => {
                    error!("unrecognized flow_atts: {:?}", v);
                    None
                }
            }
        }
        _ => None,
    }
}

pub fn get_text_atts(v: &IDLValue) -> Option<render::TextAtts> {
    match v {
        IDLValue::Record(fields) => {
            if fields.len() != 5 {
                return None;
            };
            // to do -- fix these indices;
            // get indicies from field names somehow.
            match (
                get_nat(&fields[1].val),
                get_fill(&fields[0].val),
                get_fill(&fields[3].val),
                get_dim(&fields[4].val),
                get_flow_atts(&fields[2].val),
            ) {
                (Some(zoom), Some(fg_fill), Some(bg_fill), Some(glyph_dim), Some(glyph_flow)) => {
                    Some(TextAtts {
                        zoom,
                        fg_fill,
                        bg_fill,
                        glyph_dim,
                        glyph_flow,
                    })
                }
                _ => {
                    error!("could not recognize text_atts {:?}", v);
                    None
                }
            }
        }
        _ => {
            error!("could not recognize text_atts {:?}", v);
            None
        }
    }
}

pub fn get_render_text(elm: &IDLValue) -> Option<render::Elm> {
    match elm {
        IDLValue::Record(fields) => {
            if fields.len() != 2 {
                return None;
            };
            // to do -- decompose these text atts and use them!
            match (get_text(&fields[0].val), get_text_atts(&fields[1].val)) {
                (Some(t), Some(ta)) => {
                    let mut r = Render::new();
                    r.text(&t, &ta);
                    Some(r.into_elms()[0].clone())
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn get_render_rect_fill(elm: &IDLValue) -> Option<render::Elm> {
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

pub fn get_render_node(elm: &IDLValue) -> Option<render::Elm> {
    match elm {
        IDLValue::Record(fields) => {
            if fields.len() != 3 {
                return None;
            };
            let rect = get_rect(&fields[2].val);
            let fill = get_fill(&fields[1].val);
            // to do -- decompose these text atts and use them!
            match &fields[0].val {
                IDLValue::Vec(vals) => match (rect, fill, get_render_elms(&vals)) {
                    (None, _, _) => {
                        warn!("failed to parse rect of node elm");
                        None
                    }
                    (_, None, _) => {
                        warn!("failed to parse fill of node elm");
                        None
                    }
                    (Some(rect), Some(fill), Some(node_elms)) => {
                        trace!("node with elements: {:?}", node_elms);
                        Some(render::Elm::Node(Box::new(render::Node {
                            name: Name::Void,
                            children: node_elms,
                            rect: rect,
                            fill: fill,
                        })))
                    }
                    _ => None,
                },
                _ => None,
            }
        }
        _ => None,
    }
}

/*
Variant(
    // node
    IDLField { id: 1225394690, val:
               Record(
                   [
                       // elms
                       IDLField {
                           id: 1125441421, val:
                           // elms vector
                           Vec(
                               [
                                   // #rect
                                   Variant(IDLField { id: 1269255460,
                                                      // (rect, fill)
                                                      val: Record([
                                                          // 0: rect
                                                          IDLField { id: 0, val: Record([
                                                              IDLField { id: 4996424, val: Record([
                                                                  IDLField { id: 38537191, val: Nat(10) },
                                                                  IDLField { id: 3395466758, val: Nat(10) }]) },
                                                              IDLField { id: 5594516, val: Record([
                                                                  IDLField { id: 120, val: Nat(2) },
                                                                  IDLField { id: 121, val: Nat(2) }]) }]) },
                                                          // 1: fill
                                                          IDLField { id: 1, val: Variant(
                                                              IDLField { id: 240232876, val: Record([
                                                                  IDLField { id: 0, val: Nat(0) },
                                                                  IDLField { id: 1, val: Nat(0) },
                                                                  IDLField { id: 2, val: Nat(0) }]) }) }
                                                      ])
                                   })
                               ]
                           )
                       },
                       // fill
                       IDLField { id: 1136381571, val: Variant(IDLField { id: 1225396920, val: Null }) },
                       // rect
                       IDLField { id: 1269255460, val:
                                  Record([
                                      IDLField { id: 4996424, val: Record([
                                          IDLField { id: 38537191, val: Nat(0) },
                                          IDLField { id: 3395466758, val: Nat(0) }
                                      ])
                                      },
                                      IDLField { id: 5594516, val: Record([
                                          IDLField { id: 120, val: Nat(2) },
                                          IDLField { id: 121, val: Nat(2) }]) }])
                       }
                   ]
               )
    })
*/

pub fn get_render_elm_(elm: &IDLField) -> Option<render::Elm> {
    match elm.id {
        // #node : Node
        1225394690 => get_render_node(&elm.val),
        // #text : (String, TextAtts)
        1291439277 => get_render_text(&elm.val),
        // #rect : (Rect, Fill)
        1269255460 => get_render_rect_fill(&elm.val),
        tag => {
            info!(
                "warning: recognized element tag {}, for element {:?}",
                tag, elm
            );
            None
        }
    }
}

pub fn get_render_elm(v: &IDLValue) -> Option<render::Elm> {
    // extra debugger messages here:
    match v {
        IDLValue::Variant(f) => match get_render_elm_(f) {
            None => {
                warn!("failed to parse element {:?}", v);
                return None;
            }
            Some(elm) => Some(elm),
        },
        _ => {
            warn!("unrecognized element {:?}", v);
            return None;
        }
    }
}

pub fn get_render_elms(vals: &Vec<IDLValue>) -> Option<render::Elms> {
    let mut out: render::Elms = vec![];
    for v in vals.iter() {
        match get_render_elm(v) {
            Some(elm) => out.push(elm),
            None => return None,
        }
    }
    Some(out)
}

pub fn get_render_named_elms_(vals: &Vec<IDLValue>) -> Option<render::NamedElms> {
    let mut out: render::NamedElms = vec![];
    for v in vals.iter() {
        match v {
            IDLValue::Record(fields) => {
                if fields.len() != 2 {
                    return None;
                };
                match (get_text(&fields[0].val), get_render_elm(&fields[1].val)) {
                    (Some(name), Some(elm)) => out.push((Name::Atom(Atom::String(name)), elm)),
                    _ => return None,
                };
            }
            _ => return None,
        }
    }
    Some(out)
}

pub fn get_render_named_elms(vals: &IDLValue) -> Option<render::NamedElms> {
    match vals {
        IDLValue::Vec(vals) => get_render_named_elms_(vals),
        _ => None,
    }
}

pub fn get_render_out(v: &IDLValue) -> Option<render::Out> {
    match v {
        IDLValue::Variant(field) => match field.id {
            // #draw
            1114647556 => match get_render_elm(&field.val) {
                Some(elm) => Some(render::Out::Draw(elm)),
                None => None,
            },
            // #redraw
            4271367479 => match get_render_named_elms(&field.val) {
                Some(named_elms) => Some(render::Out::Redraw(named_elms)),
                None => None,
            },
            // #renderStreams -- to do
            // otherwise
            id => {
                warn!("unexpected render_out id {:?}", id);
                None
            }
        },
        _ => None,
    }
}

pub fn get_result_render_out(v: &IDLValue) -> Option<render::Out> {
    // todo: add rendering for Ok/Err themselves
    match v {
        IDLValue::Variant(f) => match f.id {
            // #ok
            24860 => get_render_out(&f.val),
            // #err
            5048165 => get_render_out(&f.val),
            _ => {
                warn!("unexpected field {:?}", f);
                None
            }
        },
        _ => None,
    }
}

pub fn find_render_out(vs: &Vec<IDLValue>) -> Option<render::Out> {
    let mut outs: Vec<render::Out> = vec![];
    for v in vs.iter() {
        match get_result_render_out(v) {
            None => {}
            Some(o) => outs.push(o),
        }
    }
    if outs.len() == 0 {
        None
    } else if outs.len() == 1 {
        outs.pop()
    } else {
        error!("unexpected: multiple rendering outputs; using the last");
        outs.pop()
    }
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
    pub render_out: Option<render::Out>,
    pub timestamp: SystemTime,
    pub duration: Duration,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Repl {
    pub config: Config,
    pub display: Vec<(Name, render::Elm)>,
    pub history: Vec<Call>,
    // todo: log of results, parsed into MenuTree's according to the MenuType of the result type
}

impl Repl {
    pub fn update_display(&mut self, out: &Option<render::Out>) {
        match out {
            None => (), // nothing
            Some(render::Out::Draw(named_elms)) => {
                // nothing
            }
            Some(render::Out::Redraw(named_elms)) => {
                let mut cells = std::collections::HashMap::new();
                for (name, elm) in self.display.iter() {
                    cells.insert(name, elm);
                }
                for (name, elm) in named_elms.iter() {
                    cells.insert(name, elm);
                }
                let mut display: render::NamedElms = vec![];
                for (name, elm) in cells.drain().take(1) {
                    display.push((name.clone(), elm.clone()))
                }
                self.display = display;
            }
        }
    }
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
            display: vec![],
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
    eval::command_eval(&mut st, None, &cmd)?;
    Ok(st)
}

use crate::render::{FlowAtts, FrameType, Render, TextAtts};
use types::{
    lang::Dir2D,
    render::{Color, Dim, Elms, Fill},
};

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
}

fn err_atts() -> TextAtts {
    TextAtts {
        zoom: text_zoom(),
        fg_fill: Fill::Closed(Color::RGB(255, 100, 100)),
        bg_fill: Fill::None,
        glyph_dim: glyph_dim(),
        glyph_flow: glyph_flow(),
    }
}

fn msg_atts() -> TextAtts {
    TextAtts {
        zoom: text_zoom(),
        fg_fill: Fill::Closed(Color::RGB(200, 200, 255)),
        bg_fill: Fill::None,
        glyph_dim: glyph_dim(),
        glyph_flow: glyph_flow(),
    }
}

fn big_msg_atts() -> TextAtts {
    TextAtts {
        zoom: text_zoom() * 2,
        fg_fill: Fill::Closed(Color::RGB(220, 220, 255)),
        bg_fill: Fill::Closed(Color::RGB(50, 50, 50)),
        glyph_dim: glyph_dim(),
        glyph_flow: glyph_flow(),
    }
}

fn box_fill() -> Fill {
    Fill::Open(Color::RGB(50, 100, 50), 1)
}

pub fn render_elms(repl: &Repl, r: &mut Render) {
    if repl.display.len() > 0 {
        r.begin(&Name::Void, FrameType::Flow(vert_flow()));
        r.str("Render cells:", &msg_atts());
        for (name, elm) in repl.display.iter() {
            r.begin(&Name::Void, FrameType::Flow(vert_flow()));
            r.fill(Fill::Open(Color::RGB(255, 255, 255), 1));
            r.begin(&Name::Void, FrameType::Flow(horz_flow()));
            r.name(name, &data_atts());
            r.str("= ", &data_atts());
            r.end();
            r.elm(elm.clone());
            r.end();
        }
        r.end()
    }
    if repl.history.len() > 0 {
        r.begin(&Name::Void, FrameType::Flow(vert_flow()));
        r.str("Message log:", &msg_atts());
        r.begin(&Name::Void, FrameType::Flow(vert_flow()));
        r.fill(box_fill());
        for call in repl.history.iter().rev().take(10) {
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
                    match call.render_out {
                        None => {
                            if rets_idl.len() > 80 {
                                let mut rets_idl = rets_idl.clone();
                                rets_idl.truncate(80);
                                r.text(&rets_idl, &data_atts());
                                r.str("...", &data_atts())
                            } else {
                                r.text(rets_idl, &data_atts())
                            }
                        }
                        Some(ref out) => match out {
                            render::Out::Draw(elm) => r.elm(elm.clone()),
                            render::Out::Redraw(named_elms) => {
                                for (name, elm) in named_elms.iter() {
                                    r.begin(&Name::Void, FrameType::Flow(vert_flow()));
                                    r.fill(Fill::Open(Color::RGB(255, 255, 255), 1));
                                    r.begin(&Name::Void, FrameType::Flow(horz_flow()));
                                    r.name(name, &data_atts());
                                    r.str("= ", &data_atts());
                                    r.end();
                                    r.elm(elm.clone());
                                    r.end();
                                }
                            }
                        },
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
