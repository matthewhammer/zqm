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


### See also:

This project draws inspiration from many sources.

#### Computer graphics standards (2D)

At its core, ZQM is an experimental computer graphics project
that draws inspiration from existing standards for 2D computer graphics.

For instance, the [SVG standard](https://www.w3.org/TR/SVGTiny12/) shares some goals and has some
comparable concepts and parts.

As a shared goal, both ZQM and SVG offer a human-computer language
for authoring portable 2D graphics with interaction (scripting).

##### SVG standard details:

- [Scalable Vector Graphics (SVG) Tiny 1.2 Specification](https://www.w3.org/TR/SVGTiny12/)
  - [Basic data types](https://www.w3.org/TR/SVGTiny12/types.html)
  - [DOM IDL, for interactive scripts](https://www.w3.org/TR/SVGTiny12/svgudomidl.html)

ZQM aspires to produce SVG media artifacts from its own media.

#### Media/content editors/IDEs

 - https://p5stamper.com/


#### Existing "Zoom Quilts" (art projects):

The name _"Zoom Quilt Machine"_ was inspired by the experiences
of watching these art projects,
and imagining an answer to the question:
_"What tools could express the authorship of these quilts, as living data structures?"_:

 - https://www.zoomquilt.org/
 - https://zoomquilt2.com/
 - https://arkadia.xyz/
 - https://www.adultswim.com/etcetera/zoom/

