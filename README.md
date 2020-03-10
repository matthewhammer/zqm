# Zoom Quilt Machine


A [game-engine](https://en.wikipedia.org/wiki/Game_engine)-like system 
  written in [Rust](https://www.rust-lang.org/),
  for the [Internet Computer](https://dfinity.org/faq).

Initially, we focus on [PoC](https://en.wikipedia.org/wiki/Proof_of_concept) content-creation tools.

### Initial steps (done):

- [x] (_very_) basic engine: a monochrome bitmap editor.
- [x] local graphics shell (via [`Rust-SDL2`](https://github.com/Rust-SDL2/rust-sdl2)).
- [x] in-browser (client-side-only) shell via [`wasm-bindgen`](https://rustwasm.github.io/docs/wasm-bindgen/) tools.
- [x] `Candid` structure editor: Visually and interactively, edit data to store and send on the Internet Computer.

### Next steps:

- [ ] Candid support: Recognize Candid types in a file or string.
- [ ] Interactive input of text strings (e.g., for structure editor of Candid values).
- [ ] HTTP-client support: Connect to an IC canister holding saved media; send/receive media data to/from that canister.


### See also: Existing Zoom Quilts:

 - https://www.zoomquilt.org/
 - https://zoomquilt2.com/
 - https://arkadia.xyz/
 - https://www.adultswim.com/etcetera/zoom/

#### Media/content editors/IDEs
 - https://p5stamper.com/
