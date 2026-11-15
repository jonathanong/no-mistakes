use super::{programs::SameFile, provenance::RecoveredSql, LocalFunctions};
use crate::codebase::postgres::embedded::helpers::{
    program::exec::{run, Ret},
    sentinel, SiteArg,
};

impl LocalFunctions {
    pub(crate) fn identifier_clauses(&self, name: &str, args: &[SiteArg]) -> Option<Vec<String>> {
        Some(
            self.identifier_clause_values(name, args)?
                .into_iter()
                .map(|(text, _)| text)
                .collect(),
        )
    }
    pub(crate) fn identifier_clause_values(
        &self,
        name: &str,
        args: &[SiteArg],
    ) -> Option<Vec<(String, Option<RecoveredSql>)>> {
        if let Some(values) = self.guarded_identifiers.get(name) {
            return Some(values.iter().map(|text| (text.clone(), None)).collect());
        }
        let program = self.programs.get(name).cloned()?;
        let mut callees = SameFile {
            programs: &self.programs,
            depth: 0,
        };
        let mut values: Vec<(String, Option<RecoveredSql>)> = Vec::new();
        for outcome in run(&program, args, None, &mut callees)? {
            let Ret::Str(text, provenance) = outcome.ret else {
                return None;
            };
            if text.contains('\0') || sentinel::contains(&text) {
                return None;
            }
            if !values.iter().any(|(old, _)| old == &text) {
                values.push((text, provenance));
            }
        }
        (!values.is_empty()).then_some(values)
    }
}
