#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "extract_entrypoints_import_line_tests.rs"]
mod import_line_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "extract/tests/value_flow_sharing.rs"]
mod value_flow_sharing;

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "extract/tests/value_flow_once.rs"]
mod value_flow_once;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "extract/tests/no_object_mutation_literal.rs"]
mod no_object_mutation_literal;

include!("extract_import_lines.rs");
