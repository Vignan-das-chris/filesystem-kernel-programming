#![warn(missing_docs)]

extern crate bincode;
extern crate byteorder;
extern crate core;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate itertools;
extern crate libc;
#[macro_use]
extern crate log;
extern crate lz4;
extern crate owning_ref;
extern crate parking_lot;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use]
extern crate quickcheck_macros;
#[cfg(test)]
extern crate rand;
extern crate ref_slice;
extern crate seqlock;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate stable_deref_trait;
extern crate twox_hash;
#[cfg(test)]
#[macro_use]
extern crate bencher;

pub mod compression;
pub mod data_management;
pub mod database;


pub use self::database::{Database, Dataset, Error, Snapshot};
