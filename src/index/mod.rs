//! Index and storage adapters for Searus.

pub mod adapter;
pub mod memory;

pub use adapter::IndexAdapter;
pub use memory::InMemIndex;
