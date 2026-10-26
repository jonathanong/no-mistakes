// Included at the parent test scope so existing fixture helpers stay shared.
#[path = "tests/uncertainty.rs"]
mod uncertainty;

#[path = "tests/ancestry_compatibility.rs"]
mod ancestry_compatibility;

#[path = "tests/construction_targets.rs"]
mod construction_targets;
#[path = "tests/declaration_fact_boundaries.rs"]
mod declaration_fact_boundaries;
#[path = "tests/declaration_ownership.rs"]
mod declaration_ownership;
#[path = "tests/diagnostic_expressions.rs"]
mod diagnostic_expressions;
#[path = "tests/external_declarations.rs"]
mod external_declarations;
#[cfg(unix)]
#[path = "tests/symlink_declarations.rs"]
mod symlink_declarations;

#[path = "tests/constructor_proof_ownership.rs"]
mod constructor_proof_ownership;
#[path = "tests/scoped_construction_targets.rs"]
mod scoped_construction_targets;
#[path = "tests/shared_package_targets.rs"]
mod shared_package_targets;

#[path = "tests/ambient_module_targets.rs"]
mod ambient_module_targets;

#[path = "tests/constructor_identity.rs"]
mod constructor_identity;

#[path = "tests/ambient_import_equals.rs"]
mod ambient_import_equals;

#[path = "tests/constructor_target_gaps.rs"]
mod constructor_target_gaps;
#[path = "tests/declaration_proof_guards.rs"]
mod declaration_proof_guards;
#[path = "tests/virtual_module_types.rs"]
mod virtual_module_types;
#[path = "tests/workspace_export_targets.rs"]
mod workspace_export_targets;

#[path = "tests/dynamic_bases.rs"]
mod dynamic_bases;
