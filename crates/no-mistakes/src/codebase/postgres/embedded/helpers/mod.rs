//! Recovering executor SQL that is built by helper functions, including
//! helpers imported from other modules.
//!
//! Layering mirrors `relative`: the single-file walk records what it can prove
//! about the file itself (summaries of its own helpers, and the imported-helper
//! calls it composes SQL from) and the prepared resolver finishes the job later.
//! No module here reads a file, parses, or resolves an import.

mod constant;
pub(crate) mod finalize;
pub(crate) mod program;
pub(crate) mod returns;
pub(crate) mod sentinel;
mod sites;
pub(crate) mod splice;
pub(crate) mod split;
pub(crate) mod summary;
pub(crate) mod template;
mod unpublish;

pub(crate) use constant::ExportedConstant;
pub(crate) use sites::Sites;
pub(crate) use summary::{
    BuilderHelper, HelperSummary, ImportedSite, LiteralArg, PendingImportedSql, SiteArg,
};

#[cfg(test)]
mod tests;
