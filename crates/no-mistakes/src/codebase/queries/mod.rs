//! Lightweight, single-file query subcommands (issue #417).
//!
//! These trade the full structural questions of `dependencies`/`dependents`
//! for short, single-file queries an agent can reach for without formulating a
//! graph traversal. Local queries (`resolve-check`, the export list of
//! `exports-of`) only parse the target file; reverse queries (`importers`,
//! `dead-exports`, the "who imports each" of `exports-of`, and the scoping of
//! `call-sites`) build a [`SymbolIndex`] reverse import scan — cheaper than the
//! full `DepGraph`. Only `importers --tests` builds a graph, via the shared
//! test-impact engine.
//!
//! [`SymbolIndex`]: crate::codebase::dependencies::graph::SymbolIndex

pub mod call_sites;
mod call_sites_visit;
pub mod dead_exports;
pub mod exports_of;
pub mod importers;
pub mod resolve_check;

mod render;
mod reverse;
mod shared;

pub use call_sites::CallSitesArgs;
pub use dead_exports::DeadExportsArgs;
pub use exports_of::ExportsOfArgs;
pub use importers::ImportersArgs;
pub use resolve_check::ResolveCheckArgs;
