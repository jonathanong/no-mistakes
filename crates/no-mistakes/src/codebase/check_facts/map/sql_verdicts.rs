use super::CheckFactMap;
use crate::codebase::postgres::SqlObligationVerdict;
use std::sync::{Arc, Mutex};

impl CheckFactMap {
    pub fn record_sql_verdict(&self, mut verdict: SqlObligationVerdict) {
        if let Some(sink) = &self.sql_verdicts {
            verdict.application = self.sql_application.clone();
            sink.lock().unwrap().push(verdict);
        }
    }

    pub fn sql_verdicts(&self) -> Vec<SqlObligationVerdict> {
        let Some(sink) = &self.sql_verdicts else {
            return Vec::new();
        };
        let mut verdicts = sink.lock().unwrap().clone();
        verdicts.sort_by(|left, right| {
            (
                &left.profile,
                &left.rule,
                &left.application,
                &left.file,
                left.span,
                &left.subcheck,
                left.status,
                left.advisory,
                left.evidence,
                &left.known_prefix,
                &left.helper_lines,
                &left.residual_reasons,
            )
                .cmp(&(
                    &right.profile,
                    &right.rule,
                    &right.application,
                    &right.file,
                    right.span,
                    &right.subcheck,
                    right.status,
                    right.advisory,
                    right.evidence,
                    &right.known_prefix,
                    &right.helper_lines,
                    &right.residual_reasons,
                ))
        });
        verdicts
    }

    /// Report-local result sink over the same prepared source and SQL facts.
    pub fn with_fresh_sql_verdicts(&self) -> Self {
        let mut view = self.view_with_supplemental(&Self::default(), self.graph_files.clone());
        view.enable_sql_verdicts();
        view
    }

    pub fn enable_sql_verdicts(&mut self) {
        self.sql_verdicts = Some(Arc::default());
    }

    pub fn wants_sql_verdicts(&self) -> bool {
        self.sql_verdicts.is_some()
    }

    /// Borrow one prepared request through a configured rule application.
    /// The shared sink remains report-local; creating this view is opt-in.
    pub fn for_sql_application<'a>(
        &'a self,
        index: usize,
        name: Option<&'a str>,
    ) -> Option<SqlApplicationFacts<'a>> {
        let sink = self.sql_verdicts.as_deref()?;
        Some(SqlApplicationFacts {
            facts: self,
            sink,
            index,
            name,
        })
    }
}

/// Borrow prepared facts and keep verdict routing local to a rule application.
#[doc(hidden)]
pub trait SqlFactContext: Sync {
    fn prepared(&self) -> &CheckFactMap;
    fn record_sql_verdict(&self, verdict: SqlObligationVerdict);
    fn wants_sql_verdicts(&self) -> bool {
        self.prepared().wants_sql_verdicts()
    }
}

impl SqlFactContext for CheckFactMap {
    fn prepared(&self) -> &CheckFactMap {
        self
    }
    fn record_sql_verdict(&self, verdict: SqlObligationVerdict) {
        self.record_sql_verdict(verdict);
    }
}

/// Identity borrows the configuration; no prepared containers or sinks are copied.
#[doc(hidden)]
pub struct SqlApplicationFacts<'a> {
    facts: &'a CheckFactMap,
    sink: &'a Mutex<Vec<SqlObligationVerdict>>,
    index: usize,
    name: Option<&'a str>,
}

impl SqlFactContext for SqlApplicationFacts<'_> {
    fn prepared(&self) -> &CheckFactMap {
        self.facts
    }
    fn record_sql_verdict(&self, mut verdict: SqlObligationVerdict) {
        verdict.application = Some(crate::codebase::postgres::SqlRuleApplication {
            index: self.index,
            name: self.name.map(str::to_string),
        });
        self.sink.lock().unwrap().push(verdict);
    }
}
