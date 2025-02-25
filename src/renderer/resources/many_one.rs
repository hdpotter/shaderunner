// use std::collections::HashMap;

// use generational_arena::Arena;

// use crate::handle::Handle;

// use super::arena_iterator::ArenaIterator;

// pub struct ManyOne<M, O> {
//     one_to_many: HashMap<Handle<O>, Arena<Handle<M>>>,
//     many_to_one: HashMap<Handle<M>, Handle<O>>,
// }

// impl<M, O> ManyOne<M, O> {
//     pub fn new() -> Self {
//         let one_to_many = HashMap::new();
//         let many_to_one = HashMap::new();

//         Self {
//             one_to_many,
//             many_to_one,
//         }
//     }

//     pub fn one_to_many<'a>(one: Handle<O>) -> ArenaIterator<'a, O> {
//         todo!();
//     }

//     pub fn many_to_one(many: Handle<M>) -> Handle<O> {
//         todo!();
//     }

//     pub fn add_one(one: O) -> Handle<O> {
//         let arena = Arena::new();
//         one_to_many
//     }
    
//     pub fn add_many(many: M, one: O) -> Handle<M> {
//         todo!();
//     }

//     pub fn remove_many(many: Handle<M>) {
//         todo!();
//     }

//     pub fn remove_one(one: Handle<O>) {
//         todo!();
//     }
// }