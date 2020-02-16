use types::adapton::{
    Action, Closure, Context, Edge, Env, LogEvent, LogEventTag, LogEvents, Node, NodeId, Ref,
    Stack, Store, Thunk,
};
use types::lang::{Exp, Media, Name, Result as EvalResult};

/** Cleaning and dirtying algorithms.

The algorithms in this module are only used by Adapton, not
externally.  They permit the main API to dirty and clean edges while
enforcing certain invariants, given below.

### Definitions:

- An edge is either dirty or clean.

- A thunk is dirty if and only if it has at least one outgoing dirty edge.

- refs are never themselves dirty, but their dependent edges can be dirty,
  encoding the situation when the ref changes to a new value (distinct
  from at least some past action on this dirty edge).

### Clean/dirty invariant

the clean/dirty invariant for each edge is a global one, over the
status of the entire graph:

 - if an edge `E` is dirty, then all its dependent
   ("up-demand-dep"/incoming) edges are dirty too, `upFrom(E)`.

 - if an edge `E` is clean, then all of its dependencies
   ("down-demand-dep"/outgoing) edges are clean `downFrom(E)`.

These sets `upFrom(E)` and `downFrom(E)` give the transitive closure
of edges by following the dependent direction, or dependency direction
of edges, respectively.

*/

mod algo {
    use super::*;
    // todo -- rename these everywhere?
    type ThunkNode = Thunk;
    type RefNode = Ref;
    type Edges = Vec<Edge>;

    pub fn dirty_ref(ctx: &mut Context, name:&Name, ref_node:&RefNode) {
        unimplemented!()
    }

    pub fn dirty_thunk(ctx: &mut Context, name:&Name, thunk_node:&ThunkNode) {
        unimplemented!()
    }

    pub fn thunk_is_dirty(t:&ThunkNode) -> bool {
        unimplemented!()
    }

    pub fn add_edge(ctx: &mut Context, target: &NodeId, action:&Action) {
        unimplemented!()
    }

    pub fn add_back_edges(ctx: &mut Context, edges:&Edges) {
        unimplemented!()
    }

    pub fn rem_back_edges(ctx: &mut Context, edges:&Edges) {
        unimplemented!()
    }    
    
    pub fn add_back_edge(ctx: &mut Context, edge:&Edge) {
        unimplemented!()
    }

    pub fn rem_back_edge(ctx: &mut Context, edge:&Edge) {
        unimplemented!()
    }
    
    pub fn clean_edge(ctx:&mut Context, edge:&Edge) -> bool {
        unimplemented!()
    }
    
    pub fn dirty_edge(ctx: &mut Context, edge: Edge) {
        unimplemented!()
    }
}

pub fn init() -> Context {
    unimplemented!()
}

pub fn put(ctx: &mut Context, name: Name, media: Media) -> Result<NodeId, PutError> {
    unimplemented!()
}

pub fn put_thunk(
    ctx: &mut Context,
    name: Option<Name>,
    closure: Closure,
) -> Result<NodeId, PutError> {
    unimplemented!()
}

pub fn get(ctx: &mut Context, name: Name, node: NodeId) -> Result<EvalResult, GetError> {
    unimplemented!()
}

pub fn enter_scope(ctx: &mut Context, name: Name) {
    unimplemented!()
}

pub fn leave_scope(ctx: &mut Context) {
    unimplemented!()
}



pub enum PutError {}

pub enum GetError {}

impl Store {
    fn put(&mut self, name: Name, node: Node) -> Option<Node> {
        let prev = self.get(&name);
        self.0.push((name, node));
        prev
    }
    fn get(&self, name: &Name) -> Option<Node> {
        unimplemented!()
    }
}

impl Stack {
    fn push(&mut self, name: Name) {
        self.0.push(name)
    }
    fn pop(&mut self) -> Option<Name> {
        self.0.pop()
    }
}



fn begin_log_event(ctx: &mut Context) {
    let mut buf_buf = vec![];
    std::mem::swap(&mut ctx.log_buf, &mut buf_buf);
    ctx.log_stack.push(buf_buf);
}

fn log_event(tag: LogEventTag, body: LogEvents) -> LogEvent {
    match tag {
        LogEventTag::Put(n, m) => LogEvent::Put(n, m, body),
        LogEventTag::PutThunk(n, c) => LogEvent::PutThunk(n, c, body),
        LogEventTag::Get(n, c) => LogEvent::Get(n, c, body),
        LogEventTag::DirtyIncomingTo(n) => LogEvent::DirtyIncomingTo(n, body),
        LogEventTag::CleanEdgeTo(n, b) => LogEvent::CleanEdgeTo(n, b, body),
        LogEventTag::EvalThunk(n, r) => LogEvent::EvalThunk(n, r, body),
        LogEventTag::CleanThunk(n, r) => LogEvent::CleanThunk(n, r, body),
    }
}

fn end_log_event(ctx: &mut Context, tag: LogEventTag) {
    match ctx.log_stack.pop() {
        None => unreachable!(),
        Some(popped) => {
            let mut events = popped;
            std::mem::swap(&mut events, &mut ctx.log_buf);
            let event = log_event(tag, events);
            ctx.log_buf.push(event);
        }
    }
}
