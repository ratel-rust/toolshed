//! # Toolshed
//!
//! This crate contains an `Arena` allocator, along with a few common data
//! structures that can be used in tandem with it.
//!
//! For all those times when you need to create a recursively nested tree
//! of `enum`s and find yourself in pain having to put everything in
//! `Box`es all the time.

#![warn(missing_docs)]

extern crate fxhash;

pub mod cell;
pub mod map;
pub mod set;
pub mod list;
mod arena;
mod bloom;

pub use arena::Arena;
pub use cell::CopyCell;
