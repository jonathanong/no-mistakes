use super::*;
use crate::codebase::dependencies::extract::ImportedBindingKind;
use crate::codebase::rules::postgres_sql_cross_file::CompiledOptions;

pub(super) fn proven(
    opts: &CompiledOptions,
    facts: &CheckFactMap,
    bindings: &Bindings<'_>,
    call: &SqlCall<'_>,
    source: &Provenance,
) -> bool {
    let Provenance::Reference {
        name,
        scope,
        offset,
        members,
    } = source
    else {
        return false;
    };
    if members.as_slice() != ["text"] {
        return false;
    }
    let Some(binding) = bindings.value(&call.file, *scope, name, *offset) else {
        return false;
    };
    let Some(ts) = facts.get_ts_facts(&call.file) else {
        return false;
    };
    let flow = &ts.value_flow;
    if !binding.immutable
        || flow
            .registry_mutations
            .iter()
            .any(|write| write.name == *name && write.scope == *scope)
        || flow.calls.iter().any(|call| {
            matches!(&call.callee, ValueExpression::Member(base, _) if matches!(base.as_ref(),
            ValueExpression::Named { name: receiver, scope: receiver_scope, .. }
            if receiver == name && receiver_scope.unwrap_or(0) == *scope))
        })
    {
        return false;
    }
    let ValueExpression::Call(start, end) = binding.value else {
        return false;
    };
    if !flow.tagged_templates.contains(&(start, end)) {
        return false;
    }
    let Some(tag) = flow
        .calls
        .iter()
        .find(|tag| (tag.offset, tag.end) == (start, end))
    else {
        return false;
    };
    let ValueExpression::Named {
        name: local,
        scope: tag_scope,
        offset,
        ..
    } = &tag.callee
    else {
        return false;
    };
    if tag_scope.is_some_and(|scope| scope != 0)
        || bindings
            .value(&call.file, tag_scope.unwrap_or(0), local, *offset)
            .is_some()
    {
        return false;
    }
    ts.imported_bindings.iter().any(|import| {
        !import.is_type_only
            && import.local == *local
            && ((import.specifier == "sql-template-strings"
                && (import.kind == ImportedBindingKind::Default
                    || matches!(import.imported.as_str(), "default" | "SQL")))
                || opts.embedded.trusted_sql_tags.iter().any(|trusted| {
                    !trusted.module.is_empty()
                        && !trusted.name.is_empty()
                        && import.kind == ImportedBindingKind::Named
                        && import.imported == trusted.name
                        && (import.specifier == trusted.module
                            || import
                                .specifier
                                .starts_with(&format!("{}/", trusted.module)))
                }))
    })
}
