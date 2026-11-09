use super::{tokens::Tokens, *};
mod collect;
mod grouping;
mod inserts;
mod lifecycle;
pub(super) mod locations;
mod parse;
mod policy;
mod projection;
mod sources;
use crate::codebase::postgres::{parse::PreparedSql, statement_facts::SqlLocationDemand};
use collect::{collect_one, FactOut};
pub(crate) use lifecycle::project_bounds;
use lifecycle::LifecycleBuilder;
pub use parse::extract_sql_statement_facts;
pub(crate) use parse::{
    extract_from_parsed_with_recovered_placeholders, extract_sql_conflict_policy_facts,
    extract_sql_lock_scope_facts, extract_sql_statement_facts_with_bounds,
    extract_sql_statement_facts_with_json_writes, extract_sql_statement_facts_with_location_demand,
    extract_sql_statement_facts_with_policy,
    extract_sql_statement_facts_with_recovered_placeholders, extract_sql_variant_statement_facts,
    extract_with_bound_values,
};
use policy::table_keyword;
pub(crate) use sources::StatementPolicySources;
use sources::StatementSources;
use sqlparser::ast::{Spanned, Statement};
use std::{collections::HashMap, sync::Arc};
mod prepared;
use prepared::PreparedStatements;
mod extract;
use extract::extract_from_parsed_and_sources;

pub use super::not_exists::has_top_level_not_exists_in;
