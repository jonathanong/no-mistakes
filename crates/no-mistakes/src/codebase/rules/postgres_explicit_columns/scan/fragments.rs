use super::finding::{relation_message, sql_finding};
use super::fragment_ast::FragmentStar;
use super::fragment_parse::{self, FragmentContext};
use super::CompiledOptions;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::embedded::CallSqlFragment;
use crate::codebase::postgres::{EmbeddedSqlFileFacts, SchemaCatalog};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::fx::FxHashSet;
use std::path::Path;

mod advisory;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
pub(super) use advisory::{check_executed_unresolved_star, ExecutedStarContext};
use advisory::{is_placeholder_relation, record_advisory, resolved_relations};

pub(super) fn scan_embedded_fragments(
    root: &Path,
    opts: &CompiledOptions,
    embedded: &[&EmbeddedSqlFileFacts],
    catalog: &SchemaCatalog,
    sources: &SourceStore,
    prepared: &dyn SqlFactContext,
    findings: &mut crate::codebase::rules::PostgresFindings,
) {
    for file in embedded {
        if file.call_fragments.is_empty() && file.param_append_fragments.is_empty() {
            continue;
        }
        let rel = relative_slash_path(root, &file.path);
        let source =
            no_mistakes_rule_kernel::source_access::read_language_source(sources, &file.path);
        let has_unanalyzable = file.calls.iter().any(|call| call.is_unanalyzable());
        let executed_append_sites: FxHashSet<u32> = if has_unanalyzable {
            FxHashSet::default()
        } else {
            file.calls
                .iter()
                .flat_map(|call| call.variants.iter())
                .flat_map(|variant| variant.append_sites.iter().copied())
                .collect()
        };
        let mut scanner = FileScanner {
            rel: &rel,
            source: source.as_deref(),
            opts,
            catalog,
            prepared,
            seen: FxHashSet::default(),
            findings,
        };
        for span in &file.call_spans {
            if let Some(fragments) = file.call_fragments.get(span) {
                for fragment in fragments {
                    scanner.scan_fragment(fragment);
                }
            }
        }
        for fragment in &file.param_append_fragments {
            if fragment
                .append_site
                .is_none_or(|site| !executed_append_sites.contains(&site))
            {
                scanner.scan_fragment(fragment);
            }
        }
    }
}

struct FileScanner<'a> {
    rel: &'a str,
    source: Option<&'a str>,
    opts: &'a CompiledOptions,
    catalog: &'a SchemaCatalog,
    prepared: &'a dyn SqlFactContext,
    seen: FxHashSet<(u32, usize, bool, String)>,
    findings: &'a mut crate::codebase::rules::PostgresFindings,
}

impl FileScanner<'_> {
    fn scan_fragment(&mut self, fragment: &CallSqlFragment) {
        let stars =
            fragment_parse::extract_fragment_stars(&fragment.sql, fragment.context_sql.as_deref());
        if stars.is_empty() {
            return;
        }
        let context = fragment
            .context_sql
            .as_deref()
            .map(|ctx_sql| {
                if crate::codebase::postgres::embedded::helpers::sentinel::in_relation_slot(
                    ctx_sql,
                    ctx_sql.len(),
                ) {
                    fragment_parse::parse_context(&format!("{ctx_sql}{}", fragment.sql))
                } else {
                    fragment_parse::parse_context(ctx_sql)
                }
            })
            .unwrap_or_default();
        for star in &stars {
            if star.returning && !self.opts.check_returning {
                continue;
            }
            if star.within_function.as_ref().is_some_and(|func| {
                self.opts
                    .whole_row_functions
                    .iter()
                    .any(|allowed| allowed.eq_ignore_ascii_case(func))
            }) {
                continue;
            }
            let line = fragment
                .line
                .saturating_add(star.line as u32)
                .saturating_sub(1)
                .max(1) as usize;
            self.resolve_star(star, &context, line, fragment.span.0);
        }
    }

    fn resolve_star(&mut self, star: &FragmentStar, ctx: &FragmentContext, line: usize, origin: u32) {
        let (kind, verb) = if star.returning {
            ("RETURNING *", "returns")
        } else {
            ("SELECT *", "reads")
        };
        let candidates = resolved_relations(star, ctx);
        if candidates.is_empty() {
            let detail = match &star.qualifier {
                Some(q) => format!("unresolved relation `{q}` for {kind} in SQL fragment"),
                None => format!("unresolved relation for {kind} in SQL fragment"),
            };
            self.emit_advisory(origin, line, &format!("unresolved:{kind}"), star.returning, detail);
            return;
        }
        for (relation, is_physical_target) in candidates {
            if !is_physical_target
                && no_mistakes_postgres_sql::idents::relation_suffix_key(&relation) == relation
                && (ctx.ctes.iter().any(|cte| {
                    cte == &relation
                        || cte == &no_mistakes_postgres_sql::idents::relation_suffix_name(&relation)
                }) || ctx.derived_aliases.iter().any(|a| a == &relation))
            {
                continue;
            }
            if is_placeholder_relation(&relation) {
                self.emit_advisory(
                    origin,
                    line,
                    &format!("placeholder:{kind}"),
                    star.returning,
                    format!("unresolved interpolated relation for {kind} in SQL fragment"),
                );
                continue;
            }
            if let Some(text) = relation_message(kind, verb, &relation, self.opts, self.catalog) {
                if self
                    .seen
                    .insert((origin, line, star.returning, relation.clone()))
                {
                    self.findings.push(sql_finding(self.rel, line, &text, &relation));
                }
            } else if self.catalog.relation(&relation).is_none()
                && self.catalog.view_relation(&relation).is_none()
            {
                self.emit_advisory(
                    origin,
                    line,
                    &format!("unresolved:{relation}"),
                    star.returning,
                    format!("unresolved relation `{relation}` for {kind} in SQL fragment"),
                );
            }
        }
    }

    fn emit_advisory(&mut self, origin: u32, line: usize, key: &str, returning: bool, detail: String) {
        if self.seen.insert((origin, line, returning, key.to_string())) {
            record_advisory(self.prepared, self.opts, self.rel, self.source, line, returning, detail);
        }
    }
}
