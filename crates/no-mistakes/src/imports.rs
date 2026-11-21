pub use crate::fetch::imports::{
    collect_identifier_references, collect_imports, collect_imports_from_program,
    collect_runtime_imports_from_program, is_import_used, IdentifierReferenceCollector,
};
pub use crate::fetch::resolve::{relative_string, resolve_import};

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
