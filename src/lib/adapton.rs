use types::adapton::{Action, Closure, Context, Edge, Env, Node, NodeId, Ref, Thunk};
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
