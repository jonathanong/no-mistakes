//! A `sql-template-strings` tag binds every interpolation; none is raw SQL text.
use super::bindings::Bindings;
use crate::codebase::dependencies::extract::{ImportedBindingKind, ValueCall, ValueExpression};
use crate::codebase::dependencies::graph::TsFactLookup;
use std::path::Path;

const MODULE: &str = "sql-template-strings";

/// True when `value_call` is a template tagged with the `sql-template-strings`
/// default or `SQL` import. Every hole is a bind, including those of a chain
/// head or `.append(sql\`..\`)` part; raw `.append(expr)` arguments are not
/// tagged calls and stay candidates. A parameter or local with the tag's name
/// shadows the import, so it is not the library tag.
pub(super) fn is_bind_tag(bindings: &Bindings<'_>, file: &Path, value_call: &ValueCall) -> bool {
    if !matches!(value_call.arguments.first(), Some(ValueExpression::Unknown)) {
        return false;
    }
    let name = value_call.source_callee.as_str();
    if bindings.parameter(file, name) || local(bindings, file, name, &value_call.callee) {
        return false;
    }
    bindings.facts.get_ts_facts(file).is_some_and(|ts| {
        ts.imported_bindings.iter().any(|import| {
            import.specifier == MODULE
                && !import.is_type_only
                && import.local == name
                && match import.kind {
                    ImportedBindingKind::Default => true,
                    ImportedBindingKind::Named => {
                        matches!(import.imported.as_str(), "default" | "SQL")
                    }
                    ImportedBindingKind::Namespace => false,
                }
        })
    })
}

fn local(bindings: &Bindings<'_>, file: &Path, name: &str, callee: &ValueExpression) -> bool {
    matches!(callee, ValueExpression::Named { scope, offset, .. }
        if bindings.value(file, scope.unwrap_or(0), name, *offset).is_some())
}

#[cfg(test)]
mod tests;
