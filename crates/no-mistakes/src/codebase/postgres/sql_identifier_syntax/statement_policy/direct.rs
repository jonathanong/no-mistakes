//! Direct literals share a raw parse across low policies, keeping profile identity.
use super::super::{Input, Span};
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::postgres::{EmbeddedSqlOptions, SqlStatementFileFacts};
use std::{collections::BTreeMap, path::Path, sync::Arc};

pub(crate) struct Literal {
    pub facts: Arc<SqlStatementFileFacts>,
    pub mapped: bool,
}
pub(crate) type Literals = BTreeMap<(EmbeddedSqlOptions, Span), Option<Literal>>;
pub(crate) fn selected(file: &CheckFileFacts, profile: &EmbeddedSqlOptions) -> bool {
    file.sql_identifier_syntax
        .low_applications
        .iter()
        .any(|(candidate, rule, _)| candidate == profile && super::supports(rule))
}

pub(crate) fn literals<'a>(
    file: &'a CheckFileFacts,
    facts: &CheckFactMap,
    path: &Path,
    source: &str,
) -> &'a Literals {
    file.sql_identifier_syntax.policy_direct.get_or_init(|| {
        let mut prepared = BTreeMap::new();
        for (profile, embedded) in &file.embedded_sql {
            if !selected(file, profile) { continue; }
            let Ok(statements) = facts.postgres_statements(path, Some(profile)) else { continue; };
            let versions = embedded.calls.iter().zip(&embedded.call_spans)
                .flat_map(|(call, span)| call.statement_calls()
                    .filter_map(move |version| version.sql_text.clone().map(|sql| (call, *span, sql))));
            for ((call, span, sql), statement) in versions.zip(statements) {
                if call.variants.is_empty() {
                    prepared.insert((profile.clone(), span), (sql, statement));
                }
            }
        }
        let mut out = BTreeMap::new();
        for (profile, calls) in &file.sql_identifier_syntax.profiles {
            if !selected(file, profile) { continue; }
            for call in calls {
                let Input::Sql { text, source_span, holes, parameterized, .. } = &call.input else { continue; };
                out.entry((profile.clone(), *source_span)).or_insert_with(|| {
                    if !holes.is_empty() || (parameterized.is_none() && source
                        .get(source_span.0 as usize + 1..source_span.1 as usize - 1)
                        .is_none_or(|raw| !super::super::literal_projection::literal_matches(raw, text))) {
                        return None;
                    }
                    if parameterized.is_none() {
                        if let Some((sql, statement)) = prepared.get(&(profile.clone(), call.span)) {
                            if sql == text {
                                return Some(Literal { facts: Arc::new((*statement).clone()), mapped: true });
                            }
                        }
                    }
                    if let Some(observer) = crate::diagnostics::current() {
                        observer.increment("postgres.low.policy_literal_parses", 1);
                    }
                    Some(Literal {
                        facts: Arc::new(crate::codebase::postgres::statements::extract_sql_statement_facts_with_location_demand(
                            text, false, &[], crate::codebase::postgres::statement_facts::SqlLocationDemand::Full)),
                        mapped: false,
                    })
                });
            }
        }
        out
    })
}
