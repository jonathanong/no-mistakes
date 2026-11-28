use super::super::finding::relation_message;
use super::super::fragment_ast::FragmentStar;
use super::super::fragment_parse::FragmentContext;
use super::super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::statements::SqlStarProjectionFact;
use crate::codebase::postgres::{SchemaCatalog, SqlObligationVerdict, SqlVerdictStatus};
use no_mistakes_postgres_sql::idents::{relation_suffix_key, relation_suffix_name};

pub(crate) fn is_placeholder_relation(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower == "sql_dynamic_outer.column"
        || lower
            .strip_prefix("sql_placeholder_")
            .is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()))
}

pub(in super::super) struct ExecutedStarContext<'a> {
    pub(in super::super) rel: &'a str,
    pub(in super::super) source: Option<&'a str>,
    pub(in super::super) returning: bool,
    pub(in super::super) from_variant: bool,
}

pub(in super::super) fn check_executed_unresolved_star(
    prepared: &dyn SqlFactContext,
    opts: &CompiledOptions,
    catalog: &SchemaCatalog,
    ctx: ExecutedStarContext<'_>,
    star: &SqlStarProjectionFact,
) {
    let ExecutedStarContext {
        rel,
        source,
        returning,
        from_variant,
    } = ctx;
    if star.within_function.as_ref().is_some_and(|func| {
        opts.whole_row_functions
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(func))
    }) {
        return;
    }
    let kind = if returning { "RETURNING *" } else { "SELECT *" };
    if is_placeholder_relation(&star.relation) {
        record_advisory(
            prepared,
            opts,
            rel,
            source,
            star.line.max(1),
            returning,
            format!("unresolved interpolated relation for {kind}; use a closed union of table-name literals or list explicit columns"),
        );
        return;
    }
    if from_variant
        && relation_message(
            kind,
            if returning { "returns" } else { "reads" },
            &star.relation,
            opts,
            catalog,
        )
        .is_none()
        && catalog.relation(&star.relation).is_none()
        && catalog.view_relation(&star.relation).is_none()
    {
        record_advisory(
            prepared,
            opts,
            rel,
            source,
            star.line.max(1),
            returning,
            format!("unresolved relation `{}` for {kind} in SQL fragment", star.relation),
        );
    }
}

pub(super) fn resolved_relations(star: &FragmentStar, ctx: &FragmentContext) -> Vec<(String, bool)> {
    if let Some(rel) = &star.relation {
        return vec![(rel.clone(), star.returning)];
    }
    if star.returning {
        if let Some(q) = &star.qualifier {
            if let Some(target) = &ctx.mutation_target {
                if let Some(table) = find_qualified(std::slice::from_ref(target), q) {
                    return vec![(table, true)];
                }
            }
            if let Some(table) = find_qualified(&ctx.from_relations, q) {
                return vec![(table, false)];
            }
            let joined = find_join_alias(ctx, q);
            if !joined.is_empty() {
                return joined;
            }
            if ctx.ctes.iter().any(|cte| cte == q) || ctx.derived_aliases.iter().any(|a| a == q) {
                return vec![(q.clone(), false)];
            }
            return Vec::new();
        }
        return ctx
            .mutation_target
            .as_ref()
            .map(|(t, _)| vec![(t.clone(), true)])
            .unwrap_or_default();
    }
    if let Some(q) = &star.qualifier {
        if let Some(table) = find_qualified(&ctx.from_relations, q) {
            return vec![(table, false)];
        }
        let joined = find_join_alias(ctx, q);
        if !joined.is_empty() {
            return joined;
        }
        if ctx.ctes.iter().any(|cte| cte == q) || ctx.derived_aliases.iter().any(|a| a == q) {
            return vec![(q.clone(), false)];
        }
        return Vec::new();
    }
    ctx.from_relations
        .iter()
        .map(|(t, _)| (t.clone(), false))
        .collect()
}

fn find_join_alias(ctx: &FragmentContext, q: &str) -> Vec<(String, bool)> {
    ctx.join_aliases
        .iter()
        .filter(|(a, _)| a == q)
        .map(|(_, t)| (t.clone(), false))
        .collect()
}

fn find_qualified(relations: &[(String, Option<String>)], q: &str) -> Option<String> {
    relations
        .iter()
        .find(|(_, a)| a.as_deref() == Some(q))
        .or_else(|| {
            relations.iter().find(|(t, a)| {
                a.is_none()
                    && (t == q || relation_suffix_key(t) == q || relation_suffix_name(t) == q)
            })
        })
        .map(|(t, _)| t.clone())
}

pub(super) fn record_advisory(
    prepared: &dyn SqlFactContext,
    opts: &CompiledOptions,
    rel: &str,
    source: Option<&str>,
    line: usize,
    returning: bool,
    reason: String,
) {
    let span = line_span(source.unwrap_or_default(), line);
    prepared.record_sql_advisory(SqlObligationVerdict {
        profile: opts.embedded.clone(),
        rule: RULE_ID.into(),
        application: None,
        subcheck: if returning { "returning-star" } else { "select-star" }.into(),
        file: rel.into(),
        span,
        status: SqlVerdictStatus::Abstain,
        advisory: true,
        evidence: None,
        known_prefix: None,
        helper_lines: Vec::new(),
        residual_reasons: vec![reason],
    });
}

fn line_span(source: &str, line: usize) -> (u32, u32) {
    let mut current = 1usize;
    let mut start = 0usize;
    for (offset, ch) in source.char_indices() {
        if current == line {
            let end = source[offset..].find('\n').map_or(source.len(), |d| offset + d);
            return (offset as u32, end as u32);
        }
        if ch == '\n' {
            current += 1;
            start = offset + 1;
        }
    }
    (start as u32, source.len() as u32)
}
