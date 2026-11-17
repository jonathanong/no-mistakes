use super::{identity, owner, Bindings};
use crate::codebase::dependencies::extract::{
    CatalogReadKind, ValueBinding, ValueExpression, ValueFlowFacts,
};
use std::path::Path;

pub(super) fn closed(
    bindings: &Bindings<'_>,
    file: &Path,
    flow: &ValueFlowFacts,
    binding: &ValueBinding,
) -> bool {
    flow.catalogs.reads.iter().all(|(at, read)| {
        let value = ValueExpression::Named {
            name: read.name.clone(), scope: Some(read.scope), offset: *at, context: None,
        };
        if !owner(bindings, file, &value)
            .is_some_and(|other| identity(other) == identity(binding))
        {
            return true;
        }
        read.kind == CatalogReadKind::Value && (
            matches!(read.path.as_slice(), [property] if matches!(property.as_str(), "text" | "values"))
                || read.path.is_empty() && flow.bindings.iter().any(|alias| {
                    alias.immutable && alias.invalidated_at.is_none()
                        && matches!(&alias.value, ValueExpression::Named { name, scope, offset, .. }
                            if name == &read.name && scope.unwrap_or(0) == read.scope && offset == at)
                })
        )
    })
}
