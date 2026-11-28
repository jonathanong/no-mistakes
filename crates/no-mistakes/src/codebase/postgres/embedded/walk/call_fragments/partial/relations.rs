use super::partial_atom;
use crate::codebase::postgres::embedded::helpers::sentinel;
use crate::codebase::postgres::embedded::walk::ScopeVisitor;
use crate::codebase::postgres::embedded::{placeholders, tags, CallSqlFragment};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, TemplateLiteral};

pub(super) fn leaf_pieces(
    visitor: &ScopeVisitor<'_>,
    leaf: &Expression<'_>,
    combined: &[String],
    context_sql: Option<&str>,
    depth: u8,
) -> Option<Vec<String>> {
    if depth == 0 {
        return None;
    }
    let prefixes: Vec<String> = match context_sql {
        Some(ctx) => combined.iter().map(|c| format!("{ctx}{c}")).collect(),
        None => combined.to_vec(),
    };
    let expr = unwrap_ts_wrappers(leaf);
    if let Expression::TemplateLiteral(template) = expr {
        if !template.expressions.is_empty() {
            return Some(partial_templates(visitor, template, &prefixes, depth - 1));
        }
    }
    if let Some(piece) = partial_atom(visitor, expr, depth) {
        return Some(vec![piece]);
    }
    if prefixes
        .iter()
        .all(|c| sentinel::in_relation_slot(c, c.len()))
    {
        return closed_relation_names(visitor, expr);
    }
    None
}

pub(super) fn partial_templates(
    visitor: &ScopeVisitor<'_>,
    template: &TemplateLiteral<'_>,
    prefixes: &[String],
    depth: u8,
) -> Vec<String> {
    let mut versions = vec![String::new()];
    for (index, quasi) in template.quasis.iter().enumerate() {
        let text = tags::quasi_text(quasi, false);
        versions.iter_mut().for_each(|v| v.push_str(&text));
        if let Some(expr) = template.expressions.get(index) {
            if let Some(piece) = partial_atom(visitor, expr, depth) {
                versions.iter_mut().for_each(|v| v.push_str(&piece));
                continue;
            }
            let in_slot = if prefixes.is_empty() {
                versions
                    .iter()
                    .all(|v| sentinel::in_relation_slot(v, v.len()))
            } else {
                prefixes.iter().all(|p| {
                    versions.iter().all(|v| {
                        let full = format!("{p}{v}");
                        sentinel::in_relation_slot(&full, full.len())
                    })
                })
            };
            if in_slot {
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
            }
            let placeholder = placeholders::internal_placeholder(index + 1);
            versions.iter_mut().for_each(|v| v.push_str(&placeholder));
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

pub(in super::super) fn try_relation_append(
    visitor: &mut ScopeVisitor<'_>,
    key: &(usize, String),
    context: Option<&str>,
    arg: &Expression<'_>,
    straight_line: bool,
    append_start: u32,
) -> bool {
    let Some(context) = context.filter(|c| sentinel::in_relation_slot(c, c.len())) else {
        return false;
    };
    let Some(rels) = closed_relation_names(visitor, arg) else {
        return false;
    };
    let line =
        crate::codebase::ts_source::byte_offset_to_line(visitor.source, append_start as usize);
    let trace = visitor
        .call_fragment_state
        .builders
        .entry(key.clone())
        .or_default();
    apply_relation_append(trace, context, &rels, straight_line, append_start, line);
    true
}

fn apply_relation_append(
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
                    trace.alt_prefixes.clear();
                } else {
                    trace.prefix = None;
                    trace.alt_prefixes = rels.iter().map(|rel| format!("{context}{rel}")).collect();
                }
                return;
            }
        }
    }
    if rels.len() > 1 {
        trace.prefix = None;
        trace.alt_prefixes = rels.iter().map(|rel| format!("{context}{rel}")).collect();
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
