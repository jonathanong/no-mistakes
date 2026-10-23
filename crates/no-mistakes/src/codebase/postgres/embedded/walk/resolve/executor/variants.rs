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
    // A direct fluent expression owns its result. A bare Dynamic binding may
    // instead have lost its legacy proof through aliasing or an escape.
    let publication_kind =
        if kind == EmbeddedSqlKind::Dynamic && trusted_fluent_root(visitor, argument) {
            EmbeddedSqlKind::Composed
        } else {
            kind
        };
    publish(recovered, publication_kind)
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
                    versions.iter().any(|version| {
                        !version.choices.is_empty()
                            || version.variants_truncated
                            || (version.enumerated && version.append_sites.is_empty())
                    })
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

// Syntactic `.append` spelling alone cannot revive an escaped builder. Only a
// trusted inline tag or an independently retained binding proof owns the root.
fn trusted_fluent_root<'a>(visitor: &ScopeVisitor<'_>, mut value: &Expression<'a>) -> bool {
    let mut appended = false;
    loop {
        value = unwrap_ts_wrappers(value);
        let Expression::CallExpression(call) = value else {
            break;
        };
        let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
            break;
        };
        if member.property.name != "append" || call.optional {
            return false;
        }
        appended = true;
        value = &member.object;
    }
    appended
        && match value {
            Expression::TaggedTemplateExpression(tagged) => {
                crate::codebase::postgres::embedded::tags::is_sql_tag(
                    &tagged.tag,
                    &mut |name| super::super::compose::tag_shadowed(name, visitor),
                    visitor.functions.imported_sql_tags(),
                )
            }
            Expression::Identifier(id) => visitor.lookup(id.name.as_str()).is_some_and(|binding| {
                binding.sql_builder
                    && binding.sql.is_some()
                    && binding.kind != EmbeddedSqlKind::Dynamic
                    && !visitor.append_crosses_function(id.name.as_str())
            }),
            _ => false,
        }
}
