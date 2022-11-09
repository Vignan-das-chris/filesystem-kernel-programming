
use stable_deref_trait::StableDeref;
use std::fmt::{Debug, Display};
use std::hash::Hash;

pub enum ChangeKeyError<E> {
    NotPresent,
    Pinned,
    CallbackError(E),
}

impl<E> From<E> for ChangeKeyError<E> {
    fn from(e: E) -> Self {
        ChangeKeyError::CallbackError(e)
    }
}
pub enum RemoveError {
    NotPresent,
    Pinned,
}

pub trait Cache: Send + Sync {
    type Key: Eq + Hash;
    type Value;

    fn new(capacity: usize) -> Self;

    type ValueRef: AddSize + StableDeref<Target = Self::Value>;

    fn contains_key(&self, key: &Self::Key) -> bool;

    fn get(&self, key: &Self::Key, count_miss: bool) -> Option<Self::ValueRef>;

    fn remove<F>(&mut self, key: &Self::Key, f: F) -> Result<Self::Value, RemoveError>
    where
        F: FnOnce(&mut Self::Value) -> usize;

    fn force_remove(&mut self, key: &Self::Key, size: usize) -> bool;

    fn change_key<E, F>(&mut self, key: &Self::Key, f: F) -> Result<(), ChangeKeyError<E>>
    where
        F: FnOnce(
            &Self::Key,
            &mut Self::Value,
            &dyn Fn(&Self::Key) -> bool,
        ) -> Result<Self::Key, E>;

    fn force_change_key(&mut self, key: &Self::Key, new_key: Self::Key) -> bool;

    fn evict<F>(&mut self, f: F) -> Option<(Self::Key, Self::Value)>
    where
        F: FnMut(&Self::Key, &mut Self::Value, &dyn Fn(&Self::Key) -> bool) -> Option<usize>;

    fn insert(&mut self, key: Self::Key, value: Self::Value, size: usize);

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Self::Key> + 'a>;

    fn size(&self) -> usize;

    fn capacity(&self) -> usize;

    type Stats: Stats;

    fn stats(&self) -> Self::Stats;
}

pub trait AddSize: Sized {
    fn add_size(&self, size_delta: isize);

    fn finish(self, size_delta: isize) {
        self.add_size(size_delta);
    }
}

pub trait Stats: Display + Debug {
    fn capacity(&self) -> usize;
    fn size(&self) -> usize;
    fn len(&self) -> usize;
    fn hits(&self) -> u64;
    fn misses(&self) -> u64;
    fn insertions(&self) -> u64;
    fn evictions(&self) -> u64;
    fn removals(&self) -> u64;
}

mod clock;
mod clock_cache;
pub use self::clock_cache::ClockCache;
