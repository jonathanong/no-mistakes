//! Lightweight, single-file query subcommands (issue #417).
//!
//! These trade the full structural questions of `dependencies`/`dependents`
//! for short, single-file queries an agent can reach for without formulating a
//! graph traversal. Local queries (`resolve-check`, the export list of
//! `exports-of`) only parse the target file; reverse queries (`importers`,
//! `dead-exports`, and the "who imports each" of `exports-of`) project a
//! prepared [`SymbolIndex`] reverse import scan. `call-sites` projects resolved
//! callable identities from the canonical graph; `importers --tests` uses the
//! shared test-impact graph.
//!
//! [`SymbolIndex`]: crate::codebase::dependencies::graph::SymbolIndex

pub mod call_sites;
pub mod dead_exports;
pub mod exports_of;
pub mod importers;
pub mod resolve_check;

mod render;
mod reverse;
mod shared;
#[cfg(test)]
mod test_support;

pub use call_sites::CallSitesArgs;
pub use dead_exports::DeadExportsArgs;
pub use exports_of::ExportsOfArgs;
pub use importers::ImportersArgs;
pub use resolve_check::ResolveCheckArgs;
