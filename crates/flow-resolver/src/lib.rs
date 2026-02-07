//! Dependency resolution and scheduling for Agent Flow features.
//!
//! This crate provides algorithms for ordering features by their dependencies:
//! - [`topological_sort`] — Kahn's algorithm with priority-aware ordering
//! - [`find_cycles`] / [`would_create_cycle`] — cycle detection for dependency graphs
//! - [`compute_scores`] / [`are_dependencies_satisfied`] — readiness scoring
//!
//! # Example
//!
//! ```rust
//! use flow_core::Feature;
//! use flow_resolver::topological_sort;
//!
//! let features = vec![]; // your features here
//! let sorted = topological_sort(&features).unwrap();
//! ```

pub mod cycle;
pub mod scorer;
pub mod topo;

pub use cycle::{find_cycles, would_create_cycle};
pub use scorer::{are_dependencies_satisfied, compute_scores};
pub use topo::topological_sort;
