//! libclang-based call graph construction for C/C++ codebases.
//!
//! This module provides incremental, per-translation-unit call graph building
//! using libclang's AST visitor, with USR-based function identity and
//! petgraph-backed DFS/BFS traversal.
//!
//! Gated behind the `clang-graph` cargo feature.

mod compile_db;
mod compile_commands_index;
mod edge_extractor;
mod graph;
mod clang_engine;
pub mod bfs_traversal;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

pub use compile_db::CompileDbLoader;
pub use compile_commands_index::{CompileCommandsIndex, CompileArgs};
pub use edge_extractor::{CallEdge, TuParser};
pub use graph::{CallGraph, FunctionNode, TraversalOrder};
pub use clang_engine::ClangEngine;
pub use bfs_traversal::{BfsConfig, BfsPriority, BfsResult, BfsNode, BfsEdge, bfs_call_graph};
