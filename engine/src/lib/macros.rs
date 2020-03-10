macro_rules! parse_bit {
    ( $t:tt ) => {
        $t as usize != 0
    };
}

macro_rules! parse_glyph {
    ( [ $( $( $t:expr )* );* ] ) => {
        { let v =
          vec![
              $(
                  vec![
                      $(
                          parse_bit!( $t )
                      ),*
                  ]
              ),*
          ];
          let height = v.len();
          let width = v[0].len();
          for w in v.iter() {
              assert_eq!(w.len(), width);
          };
          crate::bitmap::Bitmap {
              width: width,
              height: height,
              major: crate::bitmap::Major::Row,
              bits: v
          }
        }
    };
}

macro_rules! parse_glyph_map {
    ( $( $glyph_name:tt => $glyph_value:tt ),* ) => {
        {
            let mut map = crate::std::collections::HashMap::new();
            $(
                {
                    let name  : crate::types::lang::Name = parse_name!( $glyph_name );
                    let glyph : crate::Glyph = parse_glyph!( $glyph_value );
                    map.insert(name, glyph);
                }
            )*
                map
        }
    };
}

macro_rules! parse_name {
    ( $n:ident $t:tt $m:expr) => {
        Name {
            tree: Box::new(NameTree::TaggedTuple(
                crate::types::util::name_of_string(t.to_string()),
                vec![parse_name($n); parse_name($m)],
            )),
        }
    };
    ( $n:ident ) => {
        crate::types::util::name_of_string($n.to_string())
    };
    ( $n:expr ) => {
        crate::types::util::name_of_string(format!("{}", $n))
    };
}

macro_rules! glyph_map {
    { $( $t:tt )* } => {
        pub fn glyph_map () -> crate::GlyphMap {
            let gm = parse_glyph_map!{
                $( $t )*
            };
            gm
        }
    }
}
