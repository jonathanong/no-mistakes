use super::*;

pub(super) fn named_provenance(
    facts: &CheckFactMap,
    file: &Path,
    local: &str,
    scope: Option<usize>,
    offset: u32,
    members: &[String],
) -> Provenance {
    if is_namespace_import(facts, file, local) {
        if let Some(exported) = members.first() {
            return Provenance::Namespace {
                local: local.to_string(),
                exported: exported.clone(),
                members: members.to_vec(),
            };
        }
    }
    Provenance::Reference {
        name: local.to_string(),
        scope: scope.unwrap_or(0),
        offset,
        members: members.to_vec(),
    }
}

fn is_namespace_import(facts: &CheckFactMap, file: &Path, local: &str) -> bool {
    facts.get_ts_facts(file).is_some_and(|ts| {
        ts.imported_bindings.iter().any(|binding| {
            !binding.is_type_only
                && binding.local == local
                && binding.kind == ImportedBindingKind::Namespace
        })
    })
}

pub(super) fn value_call_at<'a>(
    facts: &'a CheckFactMap,
    file: &Path,
    start: u32,
    end: u32,
) -> Option<&'a ValueCall> {
    facts
        .get_ts_facts(file)?
        .value_flow
        .calls
        .iter()
        .find(|call| call.offset == start && call.end == end)
}

pub(super) fn contains(outer: (u32, u32), inner: (u32, u32)) -> bool {
    inner.0 >= outer.0 && inner.1 <= outer.1
}
