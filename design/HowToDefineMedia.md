#### MAH/zqm -- 2020-01-13

# How to define a ZQM Media mode

## Step 1: define the media structure

Define the structure, in terms of "simplified, affine Rust"
(no references or lifetimes; everything is affine, so no `Rc<_>`s either.)


## Step 2: define `Auto` commands

Define the structure's "auto commands", as a DSL datatype.

We call these "auto commands" since their definition does not
require an editor, but only the structure being edited; we can
think of these "auto commands" as being issued by the structure
about itself, without the requirement of an editor's (cursor) state.

## Step 3: define `Editor` structure

Define a canonical editor for the structure in question.  Again, use simplified, affine Rust.

Often, this editor introduces state to track the position of a
cursor through the structure.  This definition requires a notion of
"coordinates", or some other way to define spatial location in the
structure, and a way to move it in minimal, local increments, and
(optionally) reposition it non-locally using other ways of naming
the structure's locations, perhaps even with explicit `Name`s that it embeds.

The bitmap definition here only uses 2D coordinates of type (Nat, Nat) for locations, no `Name`s.

The chain definition introduces internal `Name`s.

## Step 3a: define `EditorState`

The `EditorState` definition
  ignores the command history, and any "pre-states" before initialization completes.

## Step 3b: define `Editor`

The `Editor` definition
  includes the command history, and any "pre-states" before initialization completes.

## Step 4: Define command language ASTs.

Define the command languages, in Rust, as a system of ASTs.

## Step 4a: define `Init` commands

Define commands that initialize the editor state.

## Step 4b: define `Edit` commands

Define commands that evolve the editor state with edits,
  or changes to the edit state (cursor location).

## Step 4c: Combine languages.

Define a combined language of commands that includes (distinct) `Init`, `Auto`
and `Edit` sublanguages.

## Step 5: Implement command evaluators.

Implement the state-change semantics for the (distinct) `Init`, `Auto`
and `Edit` command languages as a system of big-step evaluators.

## Step 6: Implement system IO for the `Editor`

Implement IO for the Editor:

 - Accept input data of type Event (see types::event::Event).
 - Produce output data of type Elms (see types::render::Elms).
