use super::CheckFactMap;
use crate::codebase::postgres::SqlObligationVerdict;
use std::path::Path;

impl CheckFactMap {
    /// Internal display projection for an exactly selected low SQL application.
    /// The verdict's original identity and public serialized span stay unchanged.
    #[doc(hidden)]
    pub fn prepared_low_sql_display_line(
        &self,
        root: &Path,
        verdict: &SqlObligationVerdict,
    ) -> Option<usize> {
        let application = verdict.application.as_ref()?;
        let path = crate::codebase::ts_resolver::normalize_path(&root.join(&verdict.file));
        let syntax = &self.ts.get(&path)?.sql_identifier_syntax;
        if !syntax
            .low_applications
            .iter()
            .any(|(profile, rule, index)| {
                profile == &verdict.profile && rule == &verdict.rule && *index == application.index
            })
        {
            return None;
        }
        let calls = &syntax
            .profiles
            .iter()
            .find(|(profile, _)| profile == &verdict.profile)?
            .1;
        let index = calls
            .binary_search_by_key(&verdict.span, |call| call.span)
            .ok()?;
        Some(calls[index].line as usize)
    }
}
