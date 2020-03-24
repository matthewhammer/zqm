use serde::{Deserialize, Serialize};

use bitmap;
use glyph;
use types::{
    lang::{Atom, Dir2D, Name},
    render::{Dim, Elm, Elms, Fill, Node, Pos, Rect},
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
    pub fg_fill: Fill,
    pub bg_fill: Fill,
    pub glyph_dim: Dim,
    pub glyph_flow: FlowAtts,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct FlowAtts {
    pub dir: Dir2D,
    pub intra_pad: usize,
    pub inter_pad: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum FrameType {
    None,
    Flow(FlowAtts),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Frame {
    pub name: Name,
    pub fill: Fill,
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
                fill: Fill::None,
                elms: vec![],
                typ: FrameType::None,
            },
            stack: vec![],
        }
    }

    pub fn begin(&mut self, name: &Name, typ: FrameType) {
        let new_frame = Frame {
            name: name.clone(),
            fill: Fill::None,
            typ: typ,
            elms: vec![],
        };
        let top_frame = self.frame.clone();
        self.stack.push(top_frame);
        self.frame = new_frame;
    }

    pub fn nest(&mut self, _name: &Name, r: Render) {
        let mut r_elms = r.into_elms();
        self.frame.elms.append(&mut r_elms);
    }

    pub fn fill(&mut self, f: Fill) {
        self.frame.fill = f;
    }

    pub fn end(&mut self) {
        match self.stack.pop() {
            None => panic!("unexpected empty stack; unbalanced begin/end calls."),
            Some(saved) => {
                let cur = self.frame.clone();
                self.frame = saved;
                self.frame.elms.push(util::elm_of_frame(cur))
            }
        }
    }

    pub fn rect(&mut self, r: &Rect, f: Fill) {
        self.frame.elms.push(Elm::Rect(r.clone(), f))
    }

    pub fn bitmap(&mut self, bm: &bitmap::Bitmap, ba: &BitmapAtts) {
        let (width, height) = bitmap::semantics::bitmap_get_size(bm);
        for y in 0..height {
            for x in 0..width {
                let cell_rect = Rect::new(
                    (x * ba.zoom) as isize,
                    (y * ba.zoom) as isize,
                    ba.zoom,
                    ba.zoom,
                );
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
        assert_eq!(ta.glyph_dim.width, 5);
        assert_eq!(ta.glyph_dim.height, 5);
        let gm = glyph::cap5x5::glyph_map();
        let ba = BitmapAtts {
            zoom: ta.zoom,
            fill_isset: ta.fg_fill.clone(),
            fill_notset: ta.bg_fill.clone(),
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

    pub fn add(&mut self, elms: Elms) {
        let mut elms = elms;
        self.frame.elms.append(&mut elms)
    }

    pub fn into_elms(self) -> Elms {
        assert_eq!(self.stack.len(), 0);
        self.frame.elms
    }
}

mod util {
    use super::*;

    fn dim_of_elm(elm: &Elm) -> Dim {
        match elm {
            Elm::Node(node) => node.rect.dim.clone(),
            Elm::Rect(r, _) => r.dim.clone(),
        }
    }

    fn dim_of_flow(elms: &Elms, flow: &FlowAtts) -> Dim {
        let mut width = 0;
        let mut height = 0;
        let intra_pad_sum = flow.inter_pad * 2
            + if elms.len() == 0 {
                0
            } else {
                ((elms.len() - 1) as usize) * flow.intra_pad
            };
        match flow.dir {
            Dir2D::Left | Dir2D::Right => {
                for elm in elms.iter() {
                    let dim = dim_of_elm(elm);
                    width += dim.width;
                    height = height.max(dim.height);
                }
                width += intra_pad_sum;
                height += 2 * flow.inter_pad;
            }
            Dir2D::Up | Dir2D::Down => {
                for elm in elms.iter() {
                    let dim = dim_of_elm(elm);
                    height += dim.height;
                    width = width.max(dim.width);
                }
                height += intra_pad_sum;
                width += 2 * flow.inter_pad;
            }
        }
        let dim = Dim { width, height };
        dim
    }

    fn dim_of_frame(frame: &Frame) -> Dim {
        let dim = match frame.typ {
            FrameType::None => {
                let rect = bounding_rect_of_elms(&frame.elms);
                assert!(rect.pos.x >= 0);
                assert!(rect.pos.y >= 0);
                Dim {
                    width: (rect.pos.x as usize) + rect.dim.width,
                    height: (rect.pos.y as usize) + rect.dim.height,
                }
            }
            FrameType::Flow(ref flow) => dim_of_flow(&frame.elms, flow),
        };
        dim
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
            isize::max_value(),
            isize::max_value(),
            usize::min_value(),
            usize::min_value(),
        );
        for elm in elms.iter() {
            let rect = bounding_rect_of_elm(elm);
            bound.pos.x = min(bound.pos.x, rect.pos.x);
            bound.pos.y = min(bound.pos.y, rect.pos.y);
            bound.dim.width = max(
                bound.dim.width,
                ((rect.pos.x + rect.dim.width as isize) - bound.pos.x) as usize,
            );
            bound.dim.height = max(
                bound.dim.height,
                ((rect.pos.y + rect.dim.height as isize) - bound.pos.y) as usize,
            );
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

    fn reposition_frame_elms(frame: &Frame) -> (Elms, Rect) {
        let dim = dim_of_frame(&frame);
        let mut elms_out = vec![];
        let mut pos_out = Pos { x: 0, y: 0 };
        match frame.typ.clone() {
            FrameType::None => {
                elms_out = frame.elms.clone();
                let rect = bounding_rect_of_elms(&frame.elms);
                pos_out = rect.pos;
            }
            FrameType::Flow(flow) => {
                let p = flow.inter_pad as isize;
                let mut next_pos = match flow.dir {
                    Dir2D::Right => Pos { x: p, y: p },
                    Dir2D::Down => Pos { x: p, y: p },
                    Dir2D::Left => Pos {
                        x: p + (dim.width as isize),
                        y: p,
                    },
                    Dir2D::Up => Pos {
                        x: p,
                        y: p + (dim.height as isize),
                    },
                };
                for elm in frame.elms.iter() {
                    elms_out.push(reposition_elm(elm, next_pos.clone()));
                    let dim = dim_of_elm(elm);
                    match flow.dir {
                        Dir2D::Right => next_pos.x += (dim.width + flow.intra_pad) as isize,
                        Dir2D::Left => next_pos.x -= (dim.width + flow.intra_pad) as isize,
                        Dir2D::Down => next_pos.y += (dim.height + flow.intra_pad) as isize,
                        Dir2D::Up => next_pos.y -= (dim.height + flow.intra_pad) as isize,
                    }
                }
            }
        };
        (
            elms_out,
            Rect {
                pos: pos_out,
                dim: dim,
            },
        )
    }

    pub fn elm_of_frame(frame: Frame) -> Elm {
        let (elms, rect) = reposition_frame_elms(&frame);
        Elm::Node(Box::new(Node {
            name: frame.name,
            rect: rect,
            fill: frame.fill,
            children: elms,
        }))
    }
}
