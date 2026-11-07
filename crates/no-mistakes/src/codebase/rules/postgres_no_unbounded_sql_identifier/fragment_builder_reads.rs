use super::*;
use crate::codebase::dependencies::extract::{CatalogRead, CatalogReadKind, ValueBinding};

pub(super) fn local(
    proof: &mut Proof<'_, '_>,
    file: &Path,
    binding: &ValueBinding,
    returned: bool,
) -> bool {
    let Some(flow) = proof.file(file).map(|data| data.flow) else {
        return false;
    };
    flow.catalogs
        .reads
        .iter()
        .filter(|(_, read)| read.scope == binding.scope && read.name == binding.name)
        .all(|(at, read)| {
            readonly(read, &read.path)
                || read.path.as_slice() == ["append"] && read.kind == CatalogReadKind::Invalid
                || read.path.is_empty()
                    && (append_copy(proof, file, *at, read)
                        || returned
                            && flow.returns.iter().any(|(owner, value)| {
                                binding.owner == Some(*owner)
                                    && matches!(value,
                            ValueExpression::Named { name, scope: Some(scope), offset, .. }
                            if name == &binding.name && *scope == binding.scope && offset == at)
                            }))
        })
}

pub(super) fn readonly(read: &CatalogRead, path: &[String]) -> bool {
    matches!(path, [member] if matches!(member.as_str(), "text" | "values"))
        && read.kind == CatalogReadKind::Value
}

pub(super) fn append_copy(
    proof: &mut Proof<'_, '_>,
    file: &Path,
    at: u32,
    read: &CatalogRead,
) -> bool {
    let Some(flow) = proof.file(file).map(|data| data.flow) else {
        return false;
    };
    flow.calls.iter().any(|call| {
        !call.bound
            && call.spread_at.is_none()
            && call.arguments.len() == 1
            && matches!(&call.arguments[0], ValueExpression::Named { name, scope, offset, .. }
                if name == &read.name && scope.unwrap_or(0) == read.scope && *offset == at)
            && matches!(&call.callee, ValueExpression::Member(base, method)
                if method == "append" && fresh(proof, file, base, &mut FxHashSet::default()))
    })
}

fn fresh(
    proof: &mut Proof<'_, '_>,
    file: &Path,
    value: &ValueExpression,
    seen: &mut FxHashSet<(u32, u32)>,
) -> bool {
    let ValueExpression::Call(start, end) = value else {
        return false;
    };
    if !seen.insert((*start, *end)) {
        return false;
    }
    let Some(call) = proof
        .file(file)
        .and_then(|data| data.calls.get(&(*start, *end)))
        .copied()
    else {
        return false;
    };
    if proof.files[file]
        .flow
        .tagged_templates
        .contains_key(&(*start, *end))
    {
        return crate::codebase::rules::postgres_sql_cross_file::is_text_tag(
            proof.bindings,
            file,
            call,
            &[],
        );
    }
    !call.bound
        && call.spread_at.is_none()
        && call.arguments.len() == 1
        && matches!(&call.callee, ValueExpression::Member(base, method)
            if method == "append" && fresh(proof, file, base, seen))
}
