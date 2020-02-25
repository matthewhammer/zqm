use crate::types::render::{Elm, Elms, Fill, Pos, Rect};

pub struct Out {
    pub elms: Elms,
}
impl Out {
    pub fn new() -> Out {
        Out { elms: vec![] }
    }
    pub fn add_rect(&mut self, r: &Rect, f: Fill) {
        self.elms.push(Elm::Rect(r.clone(), f))
    }

    pub fn add_bitmap(
        &mut self,
        pos: Pos,
        bm: &bitmap::Bitmap,
        zoom: usize,
        fill_isset: Fill,
        fill_notset: Fill,
    ) {
        let mut out: Out = Out::new();
        let (width, height) = super::semantics::bitmap_get_size(bitmap);
        for x in 0..width {
            for y in 0..height {
                let cell_rect = Rect::new(x * zoom, y * zoom, zoom, zoom);
                let bit = super::semantics::bitmap_get_bit(&bitmap, x as usize, y as usize);
                let cell_fill = if bit { fill_isset } else { fill_notset };
                out.add_rect(&cell_rect, cell_fill);
            }
        }
        Ok(out.elms)
    }
    pub fn add_text(&mut self, pos: Pos, char_dim: Dim, zoom: usize, color: Color, s: String) {
        // assert char_dim is 5x5
        // loop over chars
        // for each char, add bitmap to Out and advance to next position
        // first char goes at pos; last one goes at `pos + (|s| * 5, 0)`.
        assert_eq!(char_dim.width, 5);
        assert_eq!(char_dim.height, 5);
        let gm = glyph::cap5x5::glyph_map();
        let mut x = pos.x;
        for c in s.iter() {
            match gm.find(c) {
                None => eprintf!("glyph map missing char: {:?}", c),
                Some(ref bm) => self.add_bitmap(Pos { x, y: pos.y }, bm, None, char_dim, zoom),
            }
        }
    }
}
