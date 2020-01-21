We use a special version of Adapton in this project (ZQ-Adapton) tailored to the needs of ZQM.

In several ways, ZQ-Adapton is simpler than the "fully-general" design of Adapton used by Fungi:

- Fully-general Adapton incrementalizes the host language (Rust);
  ZQ-Adapton only incrementalizes a DSL implemented in that
  language.

- Rather than have different "expressions", we only have "command
  sequences", and we do not type them (yet).

- Apart from defining new commands, we have no other mechanism for
  defining reusable functions (yet).

- Fungi introduces a general type and effects system; no
  type/effects system here (yet).  We will avoid types and
  effects until they seem useful; they are not the focus.

- Rather than have different "value types" for results of
  commands, we only ever speak of `Media`.

- Similarly, `Media` is the type of all ref cells, and all thunk
  results.  ZQM is like a simple imperative "uni-typed" language
  in this regard.

These assumptions mean that we can specialize the "engine" of
ZQ-Adapton to be an incremental evaluation layer for
the existing ZQ command language that we toggle on and off.

Toggling mechanisms:

- Global on/off switch; when on, all nodes are cached.

- Fine-grained: Properies of node names (or their hashes)
  select what nodes are cached.

- Intermediate: a new special boundry (an "Adapton Capsule")? 
  controls the timing of long-term cache effects, including eviction.
  The boundry is defined by one or more special
  name spaces (see `enter` and `leave`).
