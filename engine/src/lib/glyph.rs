/*!

Character-like bitmaps

Glyphs are bitmaps but with additional structure.

Each glyph has a (typically short) name.

Each glyph is typically small (e.g., 5x5, 6x8, etc) and corresponds to
a sign from an existing sign system, e.g., an "ASCII character",
or in some cases perhaps, novel signs for zqm.

As an extra representation invariant:
we use Major mode for all glyph definitions here.

*/

pub mod cap5x5 {

    // capital letters and additional symbols
    //
    // the first set of glyphs to bootstrap zqm's interface
    // editor: emacs (Rust major mode, in ovewrite minor mode)
    glyph_map! {
        " " => [
            0 0 0 0 0 ;
            0 0 0 0 0 ;
            0 0 0 0 0 ;
            0 0 0 0 0 ;
            0 0 0 0 0
        ],
        "**" => [
            0 0 8 0 0 ;
            0 8 8 8 0 ;
            8 8 8 8 8 ;
            0 8 8 8 0 ;
            0 0 8 0 0
        ],
        "*" => [
            0 0 0 0 0 ;
            0 0 8 0 0 ;
            0 8 0 8 0 ;
            0 0 8 0 0 ;
            0 0 0 0 0
        ],
        "0" => [
            0 8 8 8 0 ;
            8 0 0 0 8 ;
            8 0 8 0 8 ;
            8 0 0 0 8 ;
            0 8 8 8 0
        ],
        "1" => [
            0 8 8 0 0 ;
            0 0 8 0 0 ;
            0 0 8 0 0 ;
            0 0 8 0 0 ;
            0 8 8 8 0
        ],
        "2" => [
            0 8 8 8 0 ;
            8 0 0 0 8 ;
            0 0 8 8 0 ;
            0 8 0 0 0 ;
            8 8 8 8 8
        ],
        "3" => [
            0 8 8 8 0 ;
            8 0 0 0 8 ;
            0 0 8 8 0 ;
            8 0 0 0 8 ;
            0 8 8 8 0
        ],
        "4" => [
            0 0 8 0 8 ;
            0 0 8 0 8 ;
            0 8 0 0 8 ;
            8 8 8 8 8 ;
            0 0 0 0 8
        ],
        "5" => [
            8 8 8 8 8 ;
            8 0 0 0 0 ;
            0 8 8 8 0 ;
            0 0 0 0 8 ;
            8 8 8 8 0
        ],
        "6" => [
            0 8 8 8 0 ;
            8 0 0 0 0 ;
            8 8 8 8 0 ;
            8 0 0 0 8 ;
            0 8 8 8 0
        ],
        "7" => [
            8 8 8 8 8 ;
            0 0 0 0 8 ;
            0 0 0 8 0 ;
            0 0 8 0 0 ;
            0 8 0 0 0
        ],
        "8" => [
            0 8 8 8 0 ;
            8 0 0 0 8 ;
            0 8 8 8 0 ;
            8 0 0 0 8 ;
            0 8 8 8 0
        ],
        "9" => [
            0 8 8 8 0 ;
            8 0 0 0 8 ;
            0 8 8 8 8 ;
            0 0 0 0 8 ;
            0 8 8 8 0
        ],
        "a" => [
            0 0 8 0 0 ;
            0 8 0 8 0 ;
            8 0 0 0 8 ;
            8 8 8 8 8 ;
            8 0 0 0 8
        ],
        "b" => [
            8 8 8 8 0 ;
            8 0 0 0 8 ;
            8 8 8 8 0 ;
            8 0 0 0 8 ;
            8 8 8 8 0
        ],
        "c" => [
            0 8 8 8 8 ;
            8 0 0 0 0 ;
            8 0 0 0 0 ;
            8 0 0 0 0 ;
            0 8 8 8 8
        ],
        "d" => [
            8 8 8 8 0 ;
            8 0 0 0 8 ;
            8 0 0 0 8 ;
            8 0 0 0 8 ;
            8 8 8 8 0
        ],
       "e" => [
            8 8 8 8 8 ;
            8 0 0 0 0 ;
            8 8 8 0 0 ;
            8 0 0 0 0 ;
            8 8 8 8 8
        ],
        "f" => [
            8 8 8 8 8 ;
            8 0 0 0 0 ;
            8 8 8 0 0 ;
            8 0 0 0 0 ;
            8 0 0 0 0
        ],
        "g" => [
            0 8 8 8 0 ;
            8 0 0 0 0 ;
            8 0 8 8 8 ;
            8 0 0 0 8 ;
            0 8 8 8 0
        ],
        "h" => [
            8 0 0 0 8 ;
            8 0 0 0 8 ;
            8 0 0 0 8 ;
            8 8 8 8 8 ;
            8 0 0 0 8
        ],
        "i" => [
            0 8 8 8 0 ;
            0 0 8 0 0 ;
            0 0 8 0 0 ;
            0 0 8 0 0 ;
            0 8 8 8 0
        ],
        "j" => [
            0 0 0 8 8 ;
            0 0 0 0 8 ;
            0 0 0 0 8 ;
            8 0 0 0 8 ;
            0 8 8 8 0
        ],
        "k" => [
            8 0 0 0 8 ;
            8 0 0 8 0 ;
            8 8 8 0 0 ;
            8 0 0 8 0 ;
            8 0 0 0 8
        ],
        "l" => [
            8 0 0 0 0 ;
            8 0 0 0 0 ;
            8 0 0 0 0 ;
            8 0 0 0 0 ;
            8 8 8 8 8
        ],
        "m" => [
            8 0 0 0 8 ;
            8 8 0 8 8 ;
            8 0 8 0 8 ;
            8 0 0 0 8 ;
            8 0 0 0 8
        ],
        "n" => [
            8 0 0 0 8 ;
            8 8 0 0 8 ;
            8 0 8 0 8 ;
            8 0 0 8 8 ;
            8 0 0 0 8
        ],
        "o"  => [
            0 8 8 8 0 ;
            8 0 0 0 8 ;
            8 0 0 0 8 ;
            8 0 0 0 8 ;
            0 8 8 8 0
        ],
        "p" => [
            8 8 8 8 0 ;
            8 0 0 0 8 ;
            8 8 8 8 0 ;
            8 0 0 0 0 ;
            8 0 0 0 0
        ],
        "q" => [
            0 8 8 8 0 ;
            8 0 0 0 8 ;
            8 0 8 0 8 ;
            0 8 8 8 0 ;
            0 0 0 0 8
        ],
        "r" => [
            8 8 8 8 0 ;
            8 0 0 0 8 ;
            8 8 8 8 0 ;
            8 0 0 8 0 ;
            8 0 0 0 8
        ],
        "s" => [
            0 8 8 8 8 ;
            8 0 0 0 0 ;
            0 8 8 8 0 ;
            0 0 0 0 8 ;
            8 8 8 8 0
        ],
        "t" => [
            8 8 8 8 8 ;
            0 0 8 0 0 ;
            0 0 8 0 0 ;
            0 0 8 0 0 ;
            0 0 8 0 0
        ],
        "u" => [
            8 0 0 0 8 ;
            8 0 0 0 8 ;
            8 0 0 0 8 ;
            8 0 0 0 8 ;
            0 8 8 8 0
        ],
        "v" => [
            8 0 0 0 8 ;
            8 0 0 0 8 ;
            0 8 0 8 0 ;
            0 8 0 8 0 ;
            0 0 8 0 0
        ],
        "w" => [
            8 0 0 0 8 ;
            8 0 0 0 8 ;
            8 0 8 0 8 ;
            8 8 0 8 8 ;
            8 0 0 0 8
        ],
        "x" => [
            8 0 0 0 8 ;
            0 8 0 8 0 ;
            0 0 8 0 0 ;
            0 8 0 8 0 ;
            8 0 0 0 8
        ],
        "y" => [
            8 0 0 0 8 ;
            0 8 0 8 0 ;
            0 0 8 0 0 ;
            0 0 8 0 0 ;
            0 0 8 0 0
        ],
        "z" => [
            8 8 8 8 8 ;
            0 0 0 8 0 ;
            0 0 8 0 0 ;
            0 8 0 0 0 ;
            8 8 8 8 8
        ],
        " " => [
            0 0 0 0 0 ;
            0 0 0 0 0 ;
            0 0 0 0 0 ;
            0 0 0 0 0 ;
            0 0 0 0 0
        ],
        "." => [
            0 0 0 0 0 ;
            0 0 0 0 0 ;
            0 8 8 8 0 ;
            0 8 8 8 0 ;
            0 0 0 0 0
        ],
        "," => [
            0 0 0 0 0 ;
            0 0 0 0 0 ;
            0 0 0 8 0 ;
            0 0 0 8 0 ;
            0 0 8 0 0
        ],
        ";" => [
            0 8 8 8 0 ;
            0 8 8 8 0 ;
            0 0 0 0 0 ;
            0 8 8 8 0 ;
            8 8 0 0 0
        ],
        ":" => [
            0 8 8 8 0 ;
            0 8 8 8 0 ;
            0 0 0 0 0 ;
            0 8 8 8 0 ;
            0 8 8 8 0
        ],
        "<" => [
            0 0 0 8 0 ;
            0 0 8 0 0 ;
            0 8 0 0 0 ;
            0 0 8 0 0 ;
            0 0 0 8 0
        ],
        ">" => [
            0 8 0 0 0 ;
            0 0 8 0 0 ;
            0 0 0 8 0 ;
            0 0 8 0 0 ;
            0 8 0 0 0
        ],
        "(" => [
            0 0 0 8 0 ;
            0 0 8 0 0 ;
            0 0 8 0 0 ;
            0 0 8 0 0 ;
            0 0 0 8 0
        ],
        ")" => [
            0 0 8 0 0 ;
            0 0 0 8 0 ;
            0 0 0 8 0 ;
            0 0 0 8 0 ;
            0 0 8 0 0
        ],
        "{" => [
            0 0 0 8 8 ;
            0 0 8 0 0 ;
            0 8 0 8 0 ;
            0 0 8 0 0 ;
            0 0 0 8 8
        ],
        "}" => [
            8 8 0 0 0 ;
            0 0 8 0 0 ;
            0 8 0 8 0 ;
            0 0 8 0 0 ;
            8 8 0 0 0
        ],
        "]" => [
            0 0 8 8 0 ;
            0 0 0 8 0 ;
            0 0 0 8 0 ;
            0 0 0 8 0 ;
            0 0 8 8 0
        ],
        "[" => [
            0 8 8 0 0 ;
            0 8 0 0 0 ;
            0 8 0 0 0 ;
            0 8 0 0 0 ;
            0 8 8 0 0
        ],
        "\"" => [
            0 8 0 8 0 ;
            0 8 0 8 0 ;
            0 8 0 8 0 ;
            0 0 0 0 0 ;
            0 0 0 0 0
        ],
        "=" => [
            0 0 0 0 0 ;
            8 8 8 8 8 ;
            0 0 0 0 0 ;
            8 8 8 8 8 ;
            0 0 0 0 0
        ],
        "_" => [
            0 0 0 0 0 ;
            0 0 0 0 0 ;
            0 0 0 0 0 ;
            0 0 0 0 0 ;
            8 8 8 8 8
        ],
        "/" => [
            0 0 0 0 8 ;
            0 0 0 8 0 ;
            0 0 8 0 0 ;
            0 8 0 0 0 ;
            8 0 0 0 0
        ],
        "`"=> [
            0 8 0 0 0 ;
            0 0 8 0 0 ;
            0 0 0 8 0 ;
            0 0 0 0 0 ;
            0 0 0 0 0
        ],
        "#" => [
            0 8 0 8 0 ;
            8 8 8 8 8 ;
            0 8 0 8 0 ;
            8 8 8 8 8 ;
            0 8 0 8 0
        ],
        "!" => [
            0 0 0 8 8 ;
            0 0 0 8 8 ;
            0 0 8 8 0 ;
            0 0 0 0 0 ;
            0 8 8 0 0
        ]
        // todo:
        // "="
        // "{"
        // "}"
        // "["
        // "]"
    }
}
