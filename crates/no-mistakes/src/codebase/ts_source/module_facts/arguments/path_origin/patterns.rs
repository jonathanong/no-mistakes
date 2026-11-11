//! Shared binding-pattern helpers for path origins.
use super::{PathAnchor, PathOrigin};
use oxc_ast::ast::*;

/// How an object expression supplies one property.
pub(super) enum Supplied<'a> {
    /// The key is present. The value may still be an unknown path.
    Present(&'a Expression<'a>),
    /// A resolved value that is already a proven path origin.
    Proven(PathOrigin),
    /// A literal object omits the key and has no spread.
    Absent,
    /// The object shape or the value cannot be proven.
    Unknown,
}

pub(super) fn proven(origin: &PathOrigin) -> bool {
    origin.atoms().iter().all(|origin| {
        origin.anchor != PathAnchor::Unknown || origin.param.is_some() || origin.env.is_some()
    })
}

/// A parameterized helper may return a non-dynamic owned root. A dynamic return
/// drops parameter components (`join(temp, suffix)`), so it stays unknown.
pub(super) fn accept_parameterized(has_parameters: bool, origin: PathOrigin) -> Option<PathOrigin> {
    let unproven = origin
        .atoms()
        .iter()
        .any(|atom| atom.dynamic || atom.segments.iter().any(|part| part == ".."));
    if has_parameters && unproven {
        None
    } else {
        Some(origin)
    }
}

/// Key of the one direct object property that binds `symbol`.
///
/// Earlier properties are skipped. A nested, computed, or rest binding returns
/// `None` only after this symbol is found inside it.
pub(super) fn direct_property_member(
    pattern: &ObjectPattern<'_>,
    symbol: oxc_semantic::SymbolId,
) -> Option<String> {
    for property in &pattern.properties {
        if !binds_symbol(&property.value, symbol) {
            continue;
        }
        if property.computed {
            return None;
        }
        direct_binding(&property.value)?;
        let key = crate::ast::property_key_name(&property.key)?;
        return (!key.contains('.')).then(|| key.into_owned());
    }
    None
}

fn binds_symbol(pattern: &BindingPattern<'_>, symbol: oxc_semantic::SymbolId) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(binding) => binding.symbol_id.get() == Some(symbol),
        BindingPattern::AssignmentPattern(assignment) => binds_symbol(&assignment.left, symbol),
        BindingPattern::ObjectPattern(pattern) => {
            pattern
                .properties
                .iter()
                .any(|property| binds_symbol(&property.value, symbol))
                || pattern
                    .rest
                    .as_ref()
                    .is_some_and(|rest| binds_symbol(&rest.argument, symbol))
        }
        BindingPattern::ArrayPattern(pattern) => {
            pattern
                .elements
                .iter()
                .flatten()
                .any(|element| binds_symbol(element, symbol))
                || pattern
                    .rest
                    .as_ref()
                    .is_some_and(|rest| binds_symbol(&rest.argument, symbol))
        }
    }
}

/// The identifier bound by a property, plus a default expression when present.
pub(super) fn direct_binding<'a>(
    pattern: &'a BindingPattern<'a>,
) -> Option<(&'a BindingIdentifier<'a>, Option<&'a Expression<'a>>)> {
    match pattern {
        BindingPattern::BindingIdentifier(binding) => Some((binding.as_ref(), None)),
        BindingPattern::AssignmentPattern(assignment) => {
            let BindingPattern::BindingIdentifier(binding) = &assignment.left else {
                return None;
            };
            Some((binding.as_ref(), Some(&assignment.right)))
        }
        _ => None,
    }
}

pub(super) fn object_member<'a>(object: &'a ObjectExpression<'a>, key: &str) -> Supplied<'a> {
    let mut found = None;
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return Supplied::Unknown;
        };
        if property.computed || property.method || property.kind != PropertyKind::Init {
            return Supplied::Unknown;
        }
        if crate::ast::property_key_name(&property.key).as_deref() == Some(key) {
            found = Some(&property.value);
        }
    }
    match found {
        Some(value) => Supplied::Present(value),
        None => Supplied::Absent,
    }
}
