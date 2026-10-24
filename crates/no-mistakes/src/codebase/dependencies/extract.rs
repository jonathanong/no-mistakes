use crate::fx::{fx_map, fx_set, FxHashMap, FxHashSet};
use anyhow::Result;
use oxc_ast::ast::{
    AccessorProperty, Argument, ArrowFunctionExpression, AssignmentExpression, AssignmentTarget,
    BinaryExpression, BinaryOperator, BindingPattern, BlockStatement, CallExpression, CatchClause,
    Class, ClassElement, ComputedMemberExpression, Declaration, ExportAllDeclaration,
    ExportDeclaration, ExportDefaultDeclaration, ExportDefaultDeclarationKind,
    ExportFromDeclaration, ExportNamedDeclaration, ExportSpecifier, Expression, ForStatementLeft,
    FormalParameters, Function, IdentifierReference, ImportDeclaration, ImportDeclarationSpecifier,
    ImportExpression, JSXOpeningElement, MethodDefinition, MethodDefinitionKind, ModuleExportName,
    NewExpression, ObjectExpression, ObjectProperty, ObjectPropertyKind, Program,
    PropertyDefinition, PropertyKind, Statement, StaticBlock, StaticMemberExpression,
    TSEnumDeclaration, TSImportType, TSInterfaceDeclaration, TSModuleBlock, TSModuleReference,
    TSNamespaceDeclaration, TSNamespaceDeclarationBody, TSQualifiedName, TSTypeAliasDeclaration,
    TSTypeName, TSTypeParameter, TSTypeParameterDeclaration, TSTypeReference,
    TaggedTemplateExpression, VariableDeclaration, VariableDeclarationKind, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;

include!("extract_types.rs");
#[path = "extract_value_flow.rs"]
mod extract_value_flow;
include!("extract_per_item.rs");
include!("extract_replacements.rs");

include!("extract_import_facts.rs");

include!("extract_import_extractor.rs");

include!("extract_entrypoints.rs");
include!("extract_callable_assignments_finish.rs");
include!("extract_entrypoints_predeclare.rs");
include!("extract_namespace_facts.rs");
include!("extract_namespace_scan.rs");
include!("extract_namespace_locals.rs");
include!("extract_namespace_walk.rs");
include!("extract_namespace_uses.rs");
include!("extract_export_names.rs");
include!("extract_collector.rs");
include!("extract_visit_effects.rs");
include!("extract_visit_tail.rs");
include!("extract_visit_type_and_expression_methods.rs");
include!("extract_visit.rs");
include!("extract_visit_callables.rs");
include!("extract_visit_modules.rs");
include!("extract_visit_exports.rs");
include!("extract_collector_scopes.rs");
include!("extract_object_spread_helpers.rs");
include!("extract_call_apply_helpers.rs");
include!("extract_collector_methods.rs");
include!("extract_visit_members.rs");
include!("extract_visit_aggregates.rs");
include!("extract_class_heritage_helpers.rs");
include!("extract_static_getter_helpers.rs");
include!("extract_object_getter_helpers.rs");
include!("extract_class_eager_helpers.rs");
include!("extract_class_decorators.rs");
include!("extract_class_static_decorators.rs");
include!("extract_class_callable_helpers.rs");
include!("extract_visit_object_references.rs");
include!("extract_collector_aliases.rs");
#[path = "extract_collector_aggregate_aliases.rs"]
mod extract_collector_aggregate_aliases;
include!("extract_collector_deferred_aliases.rs");
include!("extract_visit_helpers.rs");
include!("extract_visit_variables.rs");
include!("extract_control_flow_scopes.rs");
include!("extract_loop_assignment_scopes.rs");
include!("extract_receiver_regions.rs");
include!("extract_execution.rs");
include!("extract_call_eligibility.rs");
include!("extract_receiver_assignment_helpers.rs");
include!("extract_default_helpers.rs");
include!("extract_object_scope_helpers.rs");
include!("extract_resource_scopes.rs");
include!("extract_type_scope_helpers.rs");
mod extract_visit_hoist;
use extract_visit_hoist::{
    predeclare_function_declarations, predeclare_hoisted_var_bindings,
    walk_arrow_function_with_body_bindings, walk_function_with_body_bindings,
};
include!("extract_visit_types.rs");
include!("extract_binding_helpers.rs");
include!("extract_syntax_helpers.rs");

#[cfg(test)]
#[path = "extract/tests/binding_and_hoist_coverage.rs"]
mod binding_and_hoist_coverage;
#[cfg(test)]
#[path = "extract/tests/collector_scope_coverage.rs"]
mod collector_scope_coverage;
#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
mod extra_tests;
#[cfg(test)]
mod import_metadata_tests;
#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "extract/ordinary_demand_tests.rs"]
mod ordinary_demand_tests;
#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "extract_statement_legacy_tests.rs"]
mod statement_legacy;
#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "extract_statement_mode.rs"]
mod statement_mode;
