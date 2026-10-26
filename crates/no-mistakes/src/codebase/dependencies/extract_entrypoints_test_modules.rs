#[cfg(test)]
#[path = "extract_entrypoints_import_line_tests.rs"]
mod import_line_tests;

#[cfg(test)]
#[path = "extract/tests/value_flow_sharing.rs"]
mod value_flow_sharing;

#[cfg(test)]
#[path = "extract/tests/value_flow_once.rs"]
mod value_flow_once;

#[cfg(test)]
#[path = "extract/tests/no_object_mutation_literal.rs"]
mod no_object_mutation_literal;

include!("extract_import_lines.rs");
