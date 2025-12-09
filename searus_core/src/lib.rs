//! Searus Core - Core types, traits, and engine for the Searus search engine.

pub mod types;
pub mod searcher;
pub mod engine;
pub mod rules;
pub mod filter;

pub mod prelude {
    //! Convenient re-exports for common types and traits.
    
    pub use crate::types::*;
    pub use crate::searcher::*;
    pub use crate::engine::*;
    pub use crate::rules::*;
    pub use crate::filter::*;
}
