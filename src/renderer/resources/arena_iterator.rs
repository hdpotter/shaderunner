use generational_arena::Arena;

use crate::handle::Handle;

pub struct ArenaIterator<'a, T> {
    iterator: generational_arena::Iter<'a, T>
}

impl<'a, T> ArenaIterator<'a, T> {
    pub fn iterate(arena: &'a Arena<T>) -> ArenaIterator<'a, T> {
        let iterator = arena.iter();
        ArenaIterator::<'a> {
            iterator,
        }
    }
}

impl<'a, T> Iterator for ArenaIterator<'a, T> {
    type Item = (Handle<T>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iterator.next() {
            Some((index, item)) => Some((Handle::new(index), item)),
            None => None,
        }
    }
}

pub struct ArenaIteratorMut<'a, T> {
    iterator: generational_arena::IterMut<'a, T>
}

impl <'a, T> ArenaIteratorMut<'a, T> {
    pub fn iterate(arena: &'a mut Arena<T>) -> ArenaIteratorMut<'a, T> {
        let iterator = arena.iter_mut();
        ArenaIteratorMut {
            iterator,
        }
    }
}

impl<'a, T> Iterator for ArenaIteratorMut<'a, T> {
    type Item = (Handle<T>, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iterator.next() {
            Some((index, item)) => Some((Handle::new(index), item)),
            None => None,
        }
    }
}