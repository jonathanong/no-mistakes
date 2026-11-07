use super::*;

impl CatalogProof<'_, '_> {
    pub(super) fn pipeline(&self, file: &Path, value: &ValueExpression) -> bool {
        let ValueExpression::Call(start, end) = value else {
            return false;
        };
        let Some(call) = self.index.calls.get(&(file, *start, *end)) else {
            return false;
        };
        let ValueExpression::Member(base, method) = &call.callee else {
            return false;
        };
        method == "join"
            && !call.bound
            && call.spread_at.is_none()
            && (call.arguments.is_empty()
                || call.arguments.len() == 1 && call.last_argument_literal)
            && self
                .list(file, base, &mut FxHashSet::default())
                .is_some_and(|(_, strings)| strings)
    }

    fn list<'a>(
        &'a self,
        file: &Path,
        value: &ValueExpression,
        seen: &mut FxHashSet<(usize, String, u32)>,
    ) -> Option<(&'a LiteralCatalog, bool)> {
        match value {
            ValueExpression::Named {
                name,
                scope: Some(scope),
                offset,
                ..
            } => {
                if !seen.insert((*scope, name.clone(), *offset)) {
                    return None;
                }
                let binding = self.bindings.value(file, *scope, name, *offset)?;
                if !binding.immutable || binding.invalidated_at.is_some() {
                    return None;
                }
                let key = (file, *scope, name.as_str());
                if self.safe_sources.contains(&key) {
                    let source = *self.index.sources.get(&key)?;
                    return (!source.record).then_some((source, false));
                }
                self.list(file, &binding.value, seen)
            }
            ValueExpression::Call(start, end) => {
                let call = self.index.calls.get(&(file, *start, *end))?;
                if call.bound || call.spread_at.is_some() || call.arguments.len() != 1 {
                    return None;
                }
                let ValueExpression::Member(base, method) = &call.callee else {
                    return None;
                };
                if !matches!(method.as_str(), "filter" | "map") {
                    return None;
                }
                let ValueExpression::Callable(owner) = &call.arguments[0] else {
                    return None;
                };
                let callback = self.index.flows.get(file)?.catalogs.callbacks.get(owner)?;
                let (source, strings) = self.list(file, base, seen)?;
                if strings {
                    return None;
                }
                let valid = callback.reads.iter().all(|read| {
                    if method == "filter" {
                        source.fields.iter().any(|field| field.starts_with(read))
                    } else {
                        source.fields.contains(read)
                    }
                });
                (valid && (method == "filter" || callback.string_output))
                    .then_some((source, method == "map"))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "pipeline_coverage_tests.rs"]
mod coverage_tests;

pub(super) fn collection_safe(
    index: &index::Index<'_>,
    file: &Path,
    source: &LiteralCatalog,
    start: u32,
    end: u32,
) -> bool {
    let Some(call) = index.calls.get(&(file, start, end)) else {
        return false;
    };
    if call.bound || call.spread_at.is_some() {
        return false;
    }
    let ValueExpression::Member(_, method) = &call.callee else {
        return false;
    };
    if method == "join" {
        return call.arguments.is_empty()
            || call.arguments.len() == 1 && call.last_argument_literal;
    }
    if !matches!(method.as_str(), "map" | "filter") || call.arguments.len() != 1 {
        return false;
    }
    let ValueExpression::Callable(owner) = &call.arguments[0] else {
        return false;
    };
    let Some(flow) = index.flows.get(file) else {
        return false;
    };
    let Some(callback) = flow.catalogs.callbacks.get(owner) else {
        return false;
    };
    if !callback.reads.iter().all(|read| {
        if method == "filter" {
            source.fields.iter().any(|field| field.starts_with(read))
        } else {
            source.fields.contains(read)
        }
    }) {
        return false;
    }
    if method == "map" {
        return callback.string_output;
    }
    let Some((start, end)) = flow.catalogs.consumers.get(&start) else {
        return false;
    };
    collection_safe(index, file, source, *start, *end)
}
