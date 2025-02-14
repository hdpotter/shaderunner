use std::marker::PhantomData;
use std::hash::{Hash, Hasher};

use generational_arena::{Arena, Index};

pub struct Handle<T> {
    index: Index,
    phantom: PhantomData<T>,
}

impl<T> Handle<T> {
    pub fn index(&self) -> Index {
        self.index
    }

    pub fn new(index: Index) -> Self {
        let phantom = PhantomData;
        
        Self {
            index,
            phantom,
        }
    }

    pub fn insert(arena: &mut Arena<T>, item: T) -> Self {
        Self::new(
            arena.insert(item)
        )
    }
}

// todo: iterator

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Handle")
            .field("index", &self.index)
            .finish()
    }
}

impl<T> Copy for Handle<T> { }

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            phantom: PhantomData,
        }
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index() == other.index()
    }
}

impl<T> Eq for Handle<T> { }

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index().hash(state);
    }
}

impl<T> std::ops::Index<Handle<T>> for Arena<T> {
    type Output = T;

    fn index(&self, index: Handle<T>) -> &Self::Output {
        &self[index.index()]
    }
}

impl<T> std::ops::IndexMut<Handle<T>> for Arena<T> {
    fn index_mut(&mut self, index: Handle<T>) -> &mut Self::Output {
        &mut self[index.index()]
    }
}