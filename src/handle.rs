use std::marker::PhantomData;
use std::hash::{Hash, Hasher};

use slotmap::{DefaultKey, HopSlotMap, SlotMap};

pub struct Handle<T> {
    key: DefaultKey,
    phantom: PhantomData<T>,
}

impl<T> Handle<T> {
    pub fn key(&self) -> DefaultKey {
        self.key
    }

    pub fn new(key: DefaultKey) -> Self {
        let phantom = PhantomData;
        
        Self {
            key,
            phantom,
        }
    }

    pub fn insert(map: &mut SlotMap<DefaultKey, T>, item: T) -> Self {
        Self::new(
            map.insert(item)
        )
    }

    pub fn insert_hop(map: &mut HopSlotMap<DefaultKey, T>, item: T) -> Self {
        Self::new(
            map.insert(item)
        )
    }
}

// todo: iterator

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Handle")
            .field("index", &self.key)
            .finish()
    }
}

impl<T> Copy for Handle<T> { }

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            phantom: PhantomData,
        }
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl<T> Eq for Handle<T> { }

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key().hash(state);
    }
}

impl<T> std::ops::Index<Handle<T>> for SlotMap<DefaultKey, T> {
    type Output = T;

    fn index(&self, index: Handle<T>) -> &Self::Output {
        &self[index.key()]
    }
}

impl<T> std::ops::IndexMut<Handle<T>> for SlotMap<DefaultKey, T> {
    fn index_mut(&mut self, index: Handle<T>) -> &mut Self::Output {
        &mut self[index.key()]
    }
}



impl<T> std::ops::Index<Handle<T>> for HopSlotMap<DefaultKey, T> {
    type Output = T;

    fn index(&self, index: Handle<T>) -> &Self::Output {
        &self[index.key()]
    }
}

impl<T> std::ops::IndexMut<Handle<T>> for HopSlotMap<DefaultKey, T> {
    fn index_mut(&mut self, index: Handle<T>) -> &mut Self::Output {
        &mut self[index.key()]
    }
}