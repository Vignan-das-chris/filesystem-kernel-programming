#![warn(missing_docs)]
#![cfg_attr(feature = "nightly", feature(const_fn))]

extern crate parking_lot;

use parking_lot::{Mutex, MutexGuard};
use std::cell::UnsafeCell;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::sync::atomic::{fence, AtomicUsize, Ordering};
use std::thread;


pub struct SeqLock<T: Copy> {
    seq: AtomicUsize,
    data: UnsafeCell<T>,
    mutex: Mutex<()>,
}

unsafe impl<T: Copy + Send> Send for SeqLock<T> {}
unsafe impl<T: Copy + Send> Sync for SeqLock<T> {}

pub struct SeqLockGuard<'a, T: Copy + 'a> {
    _guard: MutexGuard<'a, ()>,
    seqlock: &'a SeqLock<T>,
    seq: usize,
}

impl<T: Copy> SeqLock<T> {
    #[cfg(feature = "nightly")]
    #[inline]
    pub const fn new(val: T) -> SeqLock<T> {
        SeqLock {
            seq: AtomicUsize::new(0),
            data: UnsafeCell::new(val),
            mutex: Mutex::new(()),
        }
    }

    #[cfg(not(feature = "nightly"))]
    #[inline]
    pub fn new(val: T) -> SeqLock<T> {
        SeqLock {
            seq: AtomicUsize::new(0),
            data: UnsafeCell::new(val),
            mutex: Mutex::new(()),
        }
    }

    #[inline]
    pub fn read(&self) -> T {
        loop {
            let seq1 = self.seq.load(Ordering::Acquire);

            if seq1 & 1 != 0 {
                thread::yield_now();
                continue;
            }

            let result = unsafe { ptr::read_volatile(self.data.get()) };

            fence(Ordering::Acquire);

            let seq2 = self.seq.load(Ordering::Relaxed);
            if seq1 == seq2 {
                return result;
            }
        }
    }

    #[inline]
    fn begin_write(&self) -> usize {
let seq = self.seq.load(Ordering::Relaxed).wrapping_add(1);
        self.seq.store(seq, Ordering::Relaxed);

        fence(Ordering::Release);

        seq
    }

    #[inline]
    fn end_write(&self, seq: usize) {
        self.seq.store(seq.wrapping_add(1), Ordering::Release);
    }

    #[inline]
    fn lock_guard<'a>(&'a self, guard: MutexGuard<'a, ()>) -> SeqLockGuard<'a, T> {
        let seq = self.begin_write();
        SeqLockGuard {
            _guard: guard,
            seqlock: self,
            seq: seq,
        }
    }
    #[inline]
    pub fn lock_write(&self) -> SeqLockGuard<T> {
        self.lock_guard(self.mutex.lock())
    }

    #[inline]
    pub fn try_lock_write(&self) -> Option<SeqLockGuard<T>> {
        self.mutex.try_lock().map(|g| self.lock_guard(g))
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<T: Copy + Default> Default for SeqLock<T> {
    #[inline]
    fn default() -> SeqLock<T> {
        SeqLock::new(Default::default())
    }
}

impl<T: Copy + fmt::Debug> fmt::Debug for SeqLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SeqLock {{ data: {:?} }}", &self.read())
    }
}

impl<'a, T: Copy + 'a> Deref for SeqLockGuard<'a, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*self.seqlock.data.get() }
    }
}

impl<'a, T: Copy + 'a> DerefMut for SeqLockGuard<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.seqlock.data.get() }
    }
}

impl<'a, T: Copy + 'a> Drop for SeqLockGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        self.seqlock.end_write(self.seq);
    }
}
