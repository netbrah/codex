//! libclang-based call graph construction for C/C++ codebases.
//!
//! This module provides incremental, per-translation-unit call graph building
//! using libclang's AST visitor, with USR-based function identity and
//! petgraph-backed DFS/BFS traversal.
//!
//! Gated behind the `clang-graph` cargo feature.

mod compile_db;
mod edge_extractor;
mod graph;

#[cfg(test)]
mod tests;

pub use compile_db::CompileDbLoader;
pub use edge_extractor::{CallEdge, TuParser};
pub use graph::{CallGraph, TraversalOrder};
