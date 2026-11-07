use super::super::super::variants::{Recovered, ValueKind};
use super::{unwrap_ts_wrappers, EmbeddedSqlKind, Expression, ScopeVisitor};
use crate::codebase::postgres::embedded::EmbeddedSqlVariant;

pub(super) fn argument(
    visitor: &ScopeVisitor<'_>,
    argument: &Expression<'_>,
    kind: EmbeddedSqlKind,
) -> Vec<EmbeddedSqlVariant> {
    let recovered = if kind == EmbeddedSqlKind::Dynamic {
        visitor.recover_variants(argument)
    } else if kind == EmbeddedSqlKind::Composed {
        match unwrap_ts_wrappers(argument) {
            Expression::CallExpression(call)
                if matches!(unwrap_ts_wrappers(&call.callee), Expression::StaticMemberExpression(member) if member.property.name == "append")
                    || super::super::compose::contains_parameter_helper(argument, visitor) =>
            {
                visitor.recover_variants(argument)
            }
            _ => None,
        }
    } else {
        None
    };
    publish_for_argument(visitor, Some(argument), recovered, kind)
}

pub(super) fn publish_for_argument(
    visitor: &ScopeVisitor<'_>,
    argument: Option<&Expression<'_>>,
    recovered: Option<Vec<Recovered>>,
    kind: EmbeddedSqlKind,
) -> Vec<EmbeddedSqlVariant> {
    publish(
        recovered.filter(|versions| {
            !argument
                .is_some_and(|argument| revoked_static_alias(visitor, argument, kind, versions))
        }),
        kind,
    )
}

pub(super) fn publish(
    recovered: Option<Vec<Recovered>>,
    kind: EmbeddedSqlKind,
) -> Vec<EmbeddedSqlVariant> {
    recovered
        .filter(|versions| {
            versions
                .iter()
                .all(|version| version.value == ValueKind::Sql)
                && if kind == EmbeddedSqlKind::Dynamic {
                    versions
                        .iter()
                        .any(|version| version.enumerated || !version.choices.is_empty())
                } else {
                    kind == EmbeddedSqlKind::Composed
                        && versions
                            .iter()
                            .any(|version| !version.append_sites.is_empty())
                }
        })
        .unwrap_or_default()
        .into_iter()
        .map(Recovered::publish)
        .fold(Vec::new(), |mut unique, variant| {
            EmbeddedSqlVariant::push_unique(&mut unique, variant);
            unique
        })
}

// A static append can set enumerated without establishing ownership of a bare
// mutable alias. Keep the original finite/folded decision proofs independently.
fn revoked_static_alias(
    visitor: &ScopeVisitor<'_>,
    argument: &Expression<'_>,
    kind: EmbeddedSqlKind,
    versions: &[Recovered],
) -> bool {
    let Expression::Identifier(id) = unwrap_ts_wrappers(argument) else {
        return false;
    };
    kind == EmbeddedSqlKind::Dynamic
        && (visitor
            .builder_aliases
            .get(id.name.as_str())
            .is_some_and(|aliases| !aliases.is_empty())
            || visitor.append_crosses_function(id.name.as_str()))
        && versions
            .iter()
            .any(|version| !version.append_sites.is_empty())
        && versions.iter().all(|version| {
            version.choices.is_empty()
                && version.decision_offs.is_empty()
                && !version.variants_truncated
        })
}
