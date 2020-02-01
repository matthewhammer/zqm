use types::adapton::{
    Action, Closure, Context, Edge, Env, LogEvent, LogEventTag, LogEvents, Node, NodeId, Ref, Thunk,
};
use types::lang::{Exp, Media, Name, Result as EvalResult};

pub enum PutError {}

pub enum GetError {}

pub fn put(ctx: &mut Context, name: Option<Name>, media: Media) -> Result<NodeId, PutError> {
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
