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
    publish(recovered, kind)
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
