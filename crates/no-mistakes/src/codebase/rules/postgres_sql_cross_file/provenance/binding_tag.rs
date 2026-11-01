//! A `sql-template-strings` tag binds every interpolation; none is raw SQL text.
use super::super::SqlCall;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{ImportedBindingKind, ValueExpression};
use crate::codebase::dependencies::graph::TsFactLookup;

const MODULE: &str = "sql-template-strings";

/// Whether `call` can name an identifier source at all. A call that passes a
/// `sql-template-strings` template binds every hole, so it never can.
pub(crate) fn scans_identifier(facts: &CheckFactMap, call: &SqlCall<'_>) -> bool {
    super::identifier_positions::scans_identifier(call.call) && !binds_only(facts, call)
}

/// True when the executor's SQL argument is a tagged template from the
/// `sql-template-strings` default or `SQL` import. Chained `.append(raw)`,
/// plain template literals, and locals holding a template are not.
pub(crate) fn binds_only(facts: &CheckFactMap, call: &SqlCall<'_>) -> bool {
    let Some(ts) = facts.get_ts_facts(&call.file) else {
        return false;
    };
    let calls = &ts.value_flow.calls;
    let Some(outer) = calls
        .iter()
        .find(|value_call| (value_call.offset, value_call.end) == call.span)
    else {
        return false;
    };
    let Some(ValueExpression::Call(start, end)) = outer.arguments.first() else {
        return false;
    };
    let Some(tagged) = calls
        .iter()
        .find(|value_call| (value_call.offset, value_call.end) == (*start, *end))
    else {
        return false;
    };
    matches!(tagged.arguments.first(), Some(ValueExpression::Unknown))
        && ts.imported_bindings.iter().any(|import| {
            import.specifier == MODULE
                && !import.is_type_only
                && import.local == tagged.source_callee
                && match import.kind {
                    ImportedBindingKind::Default => true,
                    ImportedBindingKind::Named => {
                        matches!(import.imported.as_str(), "default" | "SQL")
                    }
                    ImportedBindingKind::Namespace => false,
                }
        })
}

#[cfg(test)]
mod tests;
