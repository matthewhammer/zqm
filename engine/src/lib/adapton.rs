use types::adapton::{
    Action, Closure, Context, Edge, LogEvent, LogEventTag, LogEvents, Node, NodeId, Ref,
    Stack, Store, Thunk,
};
use types::lang::{Media, Name, Result as EvalResult};

// See also: Adapton in Motoko:
// https://github.com/matthewhammer/cleansheets/blob/master/src/adapton.mo

mod algo {
    use super::*;
    // todo -- rename these everywhere?
    type ThunkNode = Thunk;
    type RefNode = Ref;
    type Edges = Vec<Edge>;

    #[allow(unused_variables, unused_mut)]
    pub fn dirty_ref(ctx: &mut Context, name:&Name, ref_node:&RefNode) {
        unimplemented!()
    }

    #[allow(unused_variables, unused_mut)]
    pub fn dirty_thunk(ctx: &mut Context, name:&Name, thunk_node:&ThunkNode) {
        unimplemented!()
    }

    #[allow(unused_variables, unused_mut)]
    pub fn thunk_is_dirty(t:&ThunkNode) -> bool {
        unimplemented!()
    }

    #[allow(unused_variables, unused_mut)]
    pub fn add_edge(ctx: &mut Context, target: &NodeId, action:&Action) {
        unimplemented!()
    }

    #[allow(unused_variables, unused_mut)]
    pub fn add_back_edges(ctx: &mut Context, edges:&Edges) {
        unimplemented!()
    }

    #[allow(unused_variables, unused_mut)]
    pub fn rem_back_edges(ctx: &mut Context, edges:&Edges) {
        unimplemented!()
    }

    #[allow(unused_variables, unused_mut)]
    pub fn add_back_edge(ctx: &mut Context, edge:&Edge) {
        unimplemented!()
    }

    #[allow(unused_variables, unused_mut)]
    pub fn rem_back_edge(ctx: &mut Context, edge:&Edge) {
        unimplemented!()
    }

    #[allow(unused_variables, unused_mut)]
    pub fn clean_edge(ctx:&mut Context, edge:&Edge) -> bool {
        unimplemented!()
    }

    #[allow(unused_variables, unused_mut)]
    pub fn dirty_edge(ctx: &mut Context, edge: Edge) {
        unimplemented!()
    }
}

pub fn init() -> Context {
    unimplemented!()
}

#[allow(unused_variables, unused_mut)]
pub fn put(ctx: &mut Context, name: Name, media: Media) -> Result<NodeId, PutError> {
    unimplemented!()
}

#[allow(unused_variables, unused_mut)]
pub fn put_thunk(
    ctx: &mut Context,
    name: Option<Name>,
    closure: Closure,
) -> Result<NodeId, PutError> {
    unimplemented!()
}

#[allow(unused_variables, unused_mut)]
pub fn get(ctx: &mut Context, name: Name, node: NodeId) -> Result<EvalResult, GetError> {
    unimplemented!()
}

#[allow(unused_variables, unused_mut)]
pub fn enter_scope(ctx: &mut Context, name: Name) {
    unimplemented!()
}

#[allow(unused_variables, unused_mut)]
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
    #[allow(unused_variables, unused_mut)]
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
