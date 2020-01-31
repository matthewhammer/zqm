use types::{Exp, Media, Name};
use types::adapton::{Env, Closure, Ref, Thunk, Edge, Action, NodeId, Node, Context};


pub fn enter_scope(ctx:&mut Context, name: Name) {
    unimplemented!()
}
pub fn leave_scope(ctx:&mut Context) {
    unimplemented!()
}
pub fn thunk(ctx:&mut Context, name: Option<Name>, closure: Closure) -> NodeId {
    unimplemented!()
}
pub fn set(ctx:&mut Context, name: Option<Name>, media: Media) -> NodeId {
    unimplemented!()
}
pub fn get(ctx:&mut Context, name: Name, node: NodeId) {
    unimplemented!()
}
