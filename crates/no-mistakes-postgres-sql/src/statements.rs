//! Typed INSERT, SELECT, and trigger facts from PostgreSQL SQL.
#[doc(hidden)]
pub use bounds::functions::{SCALAR as SCALAR_BUILTINS, SET_RETURNING as SET_RETURNING_BUILTINS};
#[doc(hidden)]
pub use bounds::BindValues;
pub(super) use bounds::TableTokenIndex;

mod bounds;
pub(crate) use bounds::TableTokenCursor;
mod conflict;
mod dedupe;
mod exists;
mod exists_correlation;
mod facts;
mod fallback;
mod insert;
mod insert_source;
mod json_writes;
mod limit;
mod lines;
#[doc(hidden)]
pub mod lock_writes;
mod mutations;
mod not_exists;
mod predicates;
mod row_variant;
mod select;
mod sweeps;
mod tokens;
mod transformed_predicates;
mod trigger;
pub(crate) mod value;
mod value_form;
mod wrappers;
mod writes;

pub use crate::statement_facts::*;
#[doc(hidden)]
pub use facts::locations::work;
#[doc(hidden)]
pub use facts::{
    extract_from_parsed_with_recovered_placeholders, extract_sql_conflict_policy_facts,
    extract_sql_lock_scope_facts, extract_sql_statement_facts_with_bounds,
    extract_sql_statement_facts_with_json_writes, extract_sql_statement_facts_with_location_demand,
    extract_sql_statement_facts_with_policy,
    extract_sql_statement_facts_with_recovered_placeholders, extract_sql_variant_statement_facts,
    extract_with_bound_values, project_bounds, StatementPolicySources,
};
pub use facts::{extract_sql_statement_facts, has_top_level_not_exists_in};
pub use fallback::{insert_keyword_count, mask_quoted_sql};
#[doc(hidden)]
pub use value::form_is_stable;
#[doc(hidden)]
pub use wrappers::walk_executed;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod ast_coverage_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod builtin_form_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod coverage_mask_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod coverage_more_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod coverage_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod line_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
#[cfg(test)]
mod value_ast_tests;

#[doc(hidden)]
pub use bounds::lookup_query as lookup_bound_query;
#[doc(hidden)]
pub use predicates::Projection as PredicateProjection;
#[doc(hidden)]
pub use select::{
    collect_query_projected as collect_select_query_projected, policy as select_policy,
};
