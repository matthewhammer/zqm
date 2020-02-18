# Zoom Quilt Machine


A [game-engine](https://en.wikipedia.org/wiki/Game_engine)-like system 
  written in [Rust](https://www.rust-lang.org/),
  for the [Internet Computer](https://dfinity.org/faq).

Initially, we focus on POC content-creation tools.

### Initial steps (done):

- [x] (_very_) basic engine: a monochrome bitmap editor.
- [x] local graphics shell (via [`Rust-SDL2`](https://github.com/Rust-SDL2/rust-sdl2)).
- [x] in-browser (client-side-only) shell via [`wasm-bindgen`](https://rustwasm.github.io/docs/wasm-bindgen/) tools.


### Next steps:

- [ ] Interactive input of text strings.
- [ ] Candid support: Recognize Candid types in a file or string.
- [ ] Candid structure editor: Interactively build a value of a particular Candid type.
- [ ] HTTP-client support: Connect to an IC canister holding saved media; send/receive media data to/from that canister.
