use super::partial_atom;
use crate::codebase::postgres::embedded::helpers::sentinel;
use crate::codebase::postgres::embedded::walk::ScopeVisitor;
use crate::codebase::postgres::embedded::{placeholders, tags, CallSqlFragment};
use oxc_ast::ast::{Expression, TemplateLiteral};

pub(super) fn partial_templates(
    visitor: &ScopeVisitor<'_>,
    template: &TemplateLiteral<'_>,
    depth: u8,
) -> Vec<String> {
    let mut versions = vec![String::new()];
    for (index, quasi) in template.quasis.iter().enumerate() {
        let text = tags::quasi_text(quasi, false);
        for version in &mut versions {
            version.push_str(&text);
        }
        if let Some(expr) = template.expressions.get(index) {
            if let Some(piece) = partial_atom(visitor, expr, depth) {
                for version in &mut versions {
                    version.push_str(&piece);
                }
            } else if versions
                .iter()
                .all(|version| sentinel::in_relation_slot(version, version.len()))
            {
                if let Some(rels) = closed_relation_names(visitor, expr) {
                    let cap = crate::codebase::postgres::embedded::MAX_EMBEDDED_SQL_VARIANTS;
                    if versions.len().saturating_mul(rels.len()) <= cap {
                        let mut next = Vec::with_capacity(versions.len() * rels.len());
                        for base in &versions {
                            for rel in &rels {
                                next.push(format!("{base}{rel}"));
                            }
                        }
                        versions = next;
                        continue;
                    }
                }
                let placeholder = placeholders::internal_placeholder(index + 1);
                for version in &mut versions {
                    version.push_str(&placeholder);
                }
            } else {
                let placeholder = placeholders::internal_placeholder(index + 1);
                for version in &mut versions {
                    version.push_str(&placeholder);
                }
            }
        }
    }
    versions
}

pub(in super::super) fn closed_relation_names(
    visitor: &ScopeVisitor<'_>,
    expr: &Expression<'_>,
) -> Option<Vec<String>> {
    let values =
        crate::codebase::postgres::embedded::walk::closed_values::recover_relation(visitor, expr)?;
    if values.is_empty()
        || !values.iter().all(|v| {
            !v.fragment
                && !v.sql.trim().is_empty()
                && !v.sql.contains('\0')
                && !sentinel::contains(&v.sql)
        })
    {
        return None;
    }
    Some(values.into_iter().map(|v| v.sql).collect())
}

pub(in super::super) fn apply_relation_append(
    trace: &mut super::super::BuilderTrace,
    context: &str,
    rels: &[String],
    straight_line: bool,
    append_start: u32,
    line: u32,
) {
    if straight_line {
        if let Some(last) = trace.fragments.last().cloned() {
            if sentinel::in_relation_slot(&last.sql, last.sql.len()) {
                trace.fragments.pop();
                for rel in rels {
                    let mut expanded = last.clone();
                    expanded.sql.push_str(rel);
                    trace.fragments.push(expanded);
                }
                if rels.len() == 1 {
                    if let Some(prefix) = &mut trace.prefix {
                        prefix.push_str(&rels[0]);
                    }
                } else {
                    trace.prefix = None;
                }
                return;
            }
        }
    }
    for rel in rels {
        trace.fragments.push(CallSqlFragment {
            sql: format!("{context}{rel}"),
            line,
            span: (append_start, append_start),
            context_sql: None,
            append_site: Some(append_start),
        });
    }
}

pub(crate) fn publish_call_fragments(fragments: &mut [CallSqlFragment]) {
    for fragment in fragments {
        fragment.sql = placeholders::publish_placeholders(std::mem::take(&mut fragment.sql));
        if let Some(context) = fragment.context_sql.take() {
            fragment.context_sql = Some(placeholders::publish_placeholders(context));
        }
    }
}
