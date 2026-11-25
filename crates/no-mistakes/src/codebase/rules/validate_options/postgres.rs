use super::{decode, RuleValidator};
use crate::codebase::rules;

pub(super) const VALIDATORS: &[RuleValidator] = &[
    (
        rules::postgres_constraint_validate::RULE_ID,
        decode::<rules::simple_schema_rule::Options>,
    ),
    (
        rules::postgres_fk_index::RULE_ID,
        decode::<rules::postgres_fk_index::Options>,
    ),
    (
        rules::postgres_no_add_column::RULE_ID,
        decode::<rules::postgres_no_add_column::Options>,
    ),
    (
        rules::postgres_redundant_index::RULE_ID,
        decode::<rules::postgres_redundant_index::Options>,
    ),
    (
        rules::postgres_require_fk_on_delete::RULE_ID,
        decode::<rules::simple_schema_rule::Options>,
    ),
    (
        rules::postgres_require_named_constraints::RULE_ID,
        decode::<rules::simple_schema_rule::Options>,
    ),
    (
        rules::postgres_sql_fragment_escape::RULE_ID,
        decode::<rules::postgres_sql_fragment_escape::Options>,
    ),
];
