//! Index parsed SQL obligations in this exact selected rule application.
use super::*;
use crate::codebase::postgres::{
    sql_identifier_syntax::{
        helper_recovery::{ApplicationKey, ExecutionSite},
        literal_projection::ProjectedLiteral,
        Input,
    },
    RecoveredSql,
};
use crate::fx::FxHashMap;

enum Owner<'a> {
    Direct(&'a str, (u32, u32)),
    Recovered(&'a RecoveredSql),
}
#[derive(Default)]
pub(super) struct Owners<'a> {
    sites: FxHashMap<ExecutionSite, Vec<Owner<'a>>>,
}
impl<'a> Owners<'a> {
    pub fn new(
        files: &[PathBuf],
        facts: &'a CheckFactMap,
        application: &ApplicationKey,
        sources: &SourceStore,
    ) -> Result<Self> {
        let mut owners = Self::default();
        for path in files
            .iter()
            .filter(|path| !path.extension().is_some_and(|ext| ext == "sql"))
        {
            let Some(file) = facts.ts.get(path) else {
                continue;
            };
            let Some(calls) = file.sql_identifier_syntax.calls_for(application) else {
                continue;
            };
            let source = sources.read_path(path)?;
            let literals = super::cache::literals(file, facts, path, &source);
            for call in calls {
                let site = ExecutionSite {
                    file: path.clone(),
                    start: call.span.0,
                };
                match &call.input {
                    Input::Sql {
                        text, source_span, ..
                    } if matches!(
                        literals.get(&(None, *source_span)),
                        Some(ProjectedLiteral::Direct(Ok(Some(_))))
                    ) =>
                    {
                        owners
                            .sites
                            .entry(site)
                            .or_default()
                            .push(Owner::Direct(text, *source_span))
                    }
                    Input::Recovered(_)
                        if file
                            .sql_identifier_syntax
                            .recovery
                            .sinks
                            .get(&application.profile)
                            .is_some_and(|sinks| sinks.contains(&call.span)) =>
                    {
                        if let Some(ProjectedLiteral::Recovered { alternatives }) =
                            literals.get(&(Some(application.clone()), call.span))
                        {
                            for (sql, _) in alternatives.iter().filter(|(_, facts)| facts.is_ok()) {
                                owners
                                    .sites
                                    .entry(site.clone())
                                    .or_default()
                                    .push(Owner::Recovered(sql));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(owners)
    }
    pub fn covers(
        &self,
        current: &Path,
        start: u32,
        site: &ExecutionSite,
        sql: &RecoveredSql,
    ) -> bool {
        // The canonical sink still emits; only its overlapping transport is redundant.
        if site.file == current && site.start == start {
            return false;
        }
        self.sites.get(site).is_some_and(|owners| {
            owners.iter().any(|owner| match owner {
                Owner::Direct(text, span) => {
                    *text == sql.sql
                        && !sql.origins.is_empty()
                        && sql
                            .origins
                            .iter()
                            .all(|origin| span.0 <= *origin && *origin < span.1)
                        && sql.positions.iter().all(|position| {
                            position.source_path.as_deref() == Some(site.file.as_path())
                        })
                }
                Owner::Recovered(original) => *original == sql,
            })
        })
    }
}
