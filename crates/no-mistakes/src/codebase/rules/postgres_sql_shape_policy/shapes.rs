use super::RULE_ID;
use anyhow::Result;

pub(super) const CORRELATED_EXISTS_SET_OP: &str = "correlated-exists-set-operation";
pub(super) const NOT_IN_SUBQUERY: &str = "not-in-subquery";
pub(super) const COUNT_FOR_EXISTENCE: &str = "count-for-existence";
pub(super) const LITERAL_LIMIT: &str = "literal-limit";
pub(super) const KEYSET_ONLY_SWEEP: &str = "keyset-only-sweep";
pub(super) const BANNED_FUNCTION_CALL: &str = "banned-function-call";

#[derive(Clone, Copy, Default)]
pub(crate) struct BannedShapes {
    pub(super) correlated_exists_set_operation: bool,
    pub(super) not_in_subquery: bool,
    pub(super) count_for_existence: bool,
    pub(super) literal_limit: bool,
    pub(super) keyset_only_sweep: bool,
    pub(super) banned_function_call: bool,
}

impl BannedShapes {
    pub(super) fn unanalyzable_target(&self) -> &'static str {
        if self.correlated_exists_set_operation {
            CORRELATED_EXISTS_SET_OP
        } else if self.not_in_subquery {
            NOT_IN_SUBQUERY
        } else if self.count_for_existence {
            COUNT_FOR_EXISTENCE
        } else if self.literal_limit {
            LITERAL_LIMIT
        } else if self.keyset_only_sweep {
            KEYSET_ONLY_SWEEP
        } else {
            BANNED_FUNCTION_CALL
        }
    }
}

pub(super) fn banned_shapes(values: &[String]) -> Result<BannedShapes> {
    let mut shapes = BannedShapes::default();
    let values = if values.is_empty() {
        vec![CORRELATED_EXISTS_SET_OP.to_string()]
    } else {
        values.to_vec()
    };
    for shape in &values {
        if shape.eq_ignore_ascii_case(CORRELATED_EXISTS_SET_OP) {
            shapes.correlated_exists_set_operation = true;
        } else if shape.eq_ignore_ascii_case(NOT_IN_SUBQUERY) {
            shapes.not_in_subquery = true;
        } else if shape.eq_ignore_ascii_case(COUNT_FOR_EXISTENCE) {
            shapes.count_for_existence = true;
        } else if shape.eq_ignore_ascii_case(LITERAL_LIMIT) {
            shapes.literal_limit = true;
        } else if shape.eq_ignore_ascii_case(KEYSET_ONLY_SWEEP) {
            shapes.keyset_only_sweep = true;
        } else if shape.eq_ignore_ascii_case(BANNED_FUNCTION_CALL) {
            shapes.banned_function_call = true;
        } else {
            anyhow::bail!("{RULE_ID}: unknown bannedShapes value `{shape}`");
        }
    }
    Ok(shapes)
}
