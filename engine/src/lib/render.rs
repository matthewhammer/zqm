use serde::{Deserialize, Serialize};

use bitmap;
use glyph;
use types::{
    lang::{Atom, Dir2D, Name},
    render::{Color, Dim, Elm, Elms, Fill, Node, Pos, Rect},
};

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct BitmapAtts {
    pub zoom: usize,
    pub fill_isset: Fill,
    pub fill_notset: Fill,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct TextAtts {
    pub zoom: usize,
    pub color: Color,
    pub glyph_dim: Dim,
    pub glyph_flow: FlowAtts,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct FlowAtts {
    pub dir: Dir2D,
    pub padding: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum FrameType {
    None,
    Flow(FlowAtts),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Frame {
    pub name: Name,
    pub typ: FrameType,
    pub elms: Elms,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Render {
    pub frame: Frame,
    pub stack: Vec<Frame>,
}

impl Render {
    pub fn new() -> Render {
        Render {
            frame: Frame {
                name: Name::Void,
                elms: vec![],
                typ: FrameType::None,
            },
            stack: vec![],
        }
    }

    pub fn begin(&mut self, name: &Name, typ: FrameType) {
        let new_frame = Frame {
            name: name.clone(),
            typ: typ,
            elms: vec![],
        };
        let top_frame = self.frame.clone();
        self.stack.push(top_frame);
        self.frame = new_frame;
    }

    pub fn end(&mut self) {
        match self.stack.pop() {
            None => panic!("unexpected empty stack; unbalanced begin/end calls."),
            Some(saved) => {
                let cur = self.frame.clone();
                self.frame = saved;
                self.frame.elms.push(elm_of_frame(cur))
            }
        }
    }

    pub fn rect(&mut self, r: &Rect, f: Fill) {
        self.frame.elms.push(Elm::Rect(r.clone(), f))
    }

    pub fn bitmap(&mut self, bm: &bitmap::Bitmap, ba: &BitmapAtts) {
        let (width, height) = bitmap::semantics::bitmap_get_size(bm);
        for x in 0..(width - 1) {
            for y in 0..(height - 1) {
                let cell_rect = Rect::new(x * ba.zoom, y * ba.zoom, ba.zoom, ba.zoom);
                let bit = bitmap::semantics::bitmap_get_bit(bm, x as usize, y as usize);
                let cell_fill = if bit {
                    ba.fill_isset.clone()
                } else {
                    ba.fill_notset.clone()
                };
                self.rect(&cell_rect, cell_fill);
            }
        }
    }

    pub fn atom(&mut self, atom: &Atom, ta: &TextAtts) {
        match atom {
            Atom::Bool(b) => self.text(&format!("{:?}", b), ta),
            Atom::Usize(u) => self.text(&format!("{:?}", u), ta),
            Atom::String(s) => self.text(&format!("{}", s), ta),
        }
    }

    pub fn name(&mut self, name: &Name, ta: &TextAtts) {
        match name {
            Name::Atom(a) => self.atom(a, ta),
            _ => self.text(&format!("{:?}", name), ta),
        }
    }

    pub fn str(&mut self, s: &str, ta: &TextAtts) {
        self.text(&s.to_string(), ta)
    }

    pub fn text(&mut self, text: &String, ta: &TextAtts) {
        info!("{}", text);
        assert_eq!(ta.glyph_dim.width, 5);
        assert_eq!(ta.glyph_dim.height, 5);
        let gm = glyph::cap5x5::glyph_map();
        let ba = BitmapAtts {
            zoom: ta.zoom,
            fill_isset: Fill::Closed(ta.color.clone()),
            fill_notset: Fill::None,
        };
        self.begin(&Name::Void, FrameType::Flow(ta.glyph_flow.clone()));
        for c in text.chars() {
            self.begin(&Name::Void, FrameType::None);
            match gm.get(&Name::Atom(Atom::String(c.to_string().to_lowercase()))) {
                None => eprint!("glyph map missing char: {:?}", c),
                Some(ref bm) => self.bitmap(bm, &ba),
            };
            self.end()
        }
        self.end();
    }

    pub fn into_elms(self) -> Elms {
        assert_eq!(self.stack.len(), 0);
        self.frame.elms
    }
}

fn bounding_rect_of_elm(elm: &Elm) -> Rect {
    match elm {
        Elm::Node(node) => node.rect.clone(),
        Elm::Rect(r, _) => r.clone(),
    }
}

fn bounding_rect_of_elms(elms: &Elms) -> Rect {
    use std::cmp::{max, min};
    let mut bound = Rect::new(
        usize::max_value(),
        usize::max_value(),
        usize::min_value(),
        usize::min_value(),
    );
    for elm in elms.iter() {
        let rect = bounding_rect_of_elm(elm);
        bound.pos.x = min(bound.pos.x, rect.pos.x);
        bound.pos.y = min(bound.pos.y, rect.pos.y);
        bound.dim.width = max(bound.dim.width, bound.pos.x + rect.dim.width);
        bound.dim.height = max(bound.dim.height, bound.pos.y + rect.dim.height);
    }
    bound
}

fn reposition_rect(rect: &Rect, pos: Pos) -> Rect {
    Rect {
        pos,
        dim: rect.dim.clone(),
    }
}

fn reposition_elm(elm: &Elm, pos: Pos) -> Elm {
    match elm {
        Elm::Rect(r, f) => Elm::Rect(reposition_rect(r, pos), f.clone()),
        Elm::Node(node) => Elm::Node(Box::new(Node {
            rect: reposition_rect(&node.rect, pos),
            ..*node.clone()
        })),
    }
}

fn reposition_elms(elms: &Elms, flow: FlowAtts) -> (Elms, Rect) {
    let mut next_pos = Pos { x: 0, y: 0 };
    let mut elms_out = vec![];
    for elm in elms.iter() {
        elms_out.push(reposition_elm(elm, next_pos.clone()));
        next_pos.x += bounding_rect_of_elm(elm).dim.width;
    }
    let rect_out = bounding_rect_of_elms(&elms_out);
    (elms_out, rect_out)
}

fn elm_of_elms(name: Name, elms: Elms, rect: Rect) -> Elm {
    fn node_of_elms(name: Name, elms: Elms, rect: Rect) -> Node {
        Node {
            name: name,
            rect: rect,
            fill: Fill::None,
            children: elms,
        }
    };
    Elm::Node(Box::new(node_of_elms(name, elms, rect)))
}

fn elm_of_frame(frame: Frame) -> Elm {
    match frame.typ {
        FrameType::None => {
            let rect = bounding_rect_of_elms(&frame.elms);
            elm_of_elms(frame.name, frame.elms, rect)
        }
        FrameType::Flow(flow) => {
            let (elms, rect) = reposition_elms(&frame.elms, flow);
            elm_of_elms(frame.name, elms, rect)
        }
    }
}
