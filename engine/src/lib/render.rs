use crate::types::render::{Rect, Elm, Elms, Fill};

pub struct Out {
    pub elms:Elms
}
impl Out {
    pub fn new() -> Out {
        Out{ elms: vec![] }
    }
    pub fn add_rect(&mut self, r:&Rect, f:Fill) {
        self.elms.push(Elm::Rect(r.clone(), f))
    }
}
