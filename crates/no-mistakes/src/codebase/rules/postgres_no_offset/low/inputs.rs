//! Input projection keeps parsing uncertainty independent of definite findings.
use super::*;
use crate::codebase::postgres::{
    SqlOffsetFact,
    sql_identifier_syntax::{
        Input, OffsetLiteral,
        helper_recovery::{ApplicationKey, Recovered},
        literal_projection::ProjectedLiteral,
    },
};
use std::collections::BTreeMap;

type Literals = BTreeMap<(Option<ApplicationKey>, (u32, u32)), OffsetLiteral>;
#[derive(Default)]
pub(super) struct Outcome {
    pub unknown: bool,
    pub invalid: bool,
    pub violation: bool,
}
pub(super) struct Inputs<'a> {
    pub root: &'a Path,
    pub path: &'a Path,
    pub rel: &'a str,
    pub source: &'a str,
    pub literals: &'a Literals,
}
impl Inputs<'_> {
    pub fn evaluate(
        &self,
        call: &SqlCall,
        application: &ApplicationKey,
        ordinal: &mut usize,
        findings: &mut PostgresFindings,
    ) -> Outcome {
        match &call.input {
            Input::Sql { source_span, .. } => self.direct(call, *source_span, ordinal, findings),
            Input::Recovered(proof) => self.recovered(call, application, proof, ordinal, findings),
            _ => Outcome {
                unknown: true,
                ..Default::default()
            },
        }
    }
    fn direct(
        &self,
        call: &SqlCall,
        span: (u32, u32),
        ordinal: &mut usize,
        findings: &mut PostgresFindings,
    ) -> Outcome {
        let Some(ProjectedLiteral::Direct(result)) = self.literals.get(&(None, span)) else {
            return Outcome {
                unknown: true,
                ..Default::default()
            };
        };
        let mut result_flags = Outcome {
            unknown: !matches!(result, Ok(Some(_))),
            invalid: result.is_err(),
            violation: false,
        };
        if let Ok(Some(offsets)) = result {
            result_flags.violation = !offsets.is_empty();
            for offset in offsets {
                self.emit(call, *offset, self.path, ordinal, findings);
            }
        }
        result_flags
    }
    fn recovered(
        &self,
        call: &SqlCall,
        application: &ApplicationKey,
        proof: &Recovered,
        ordinal: &mut usize,
        findings: &mut PostgresFindings,
    ) -> Outcome {
        let mut result = Outcome::default();
        match self.literals.get(&(Some(application.clone()), call.span)) {
            Some(ProjectedLiteral::Recovered { alternatives, .. }) => {
                for (sql, facts) in alternatives {
                    match facts {
                        Ok(offsets) => {
                            result.violation |= !offsets.is_empty();
                            for offset in offsets {
                                let (origin, line) =
                                    Recovered::position(sql, offset.line, offset.column, self.path);
                                let mut offset = *offset;
                                offset.line = line;
                                self.emit(call, offset, origin, ordinal, findings);
                            }
                        }
                        Err(()) => result.invalid = true,
                    }
                }
            }
            _ => result.invalid = true,
        }
        result.unknown = !proof.complete() || result.invalid;
        result
    }
    fn emit(
        &self,
        call: &SqlCall,
        mut offset: SqlOffsetFact,
        origin: &Path,
        ordinal: &mut usize,
        findings: &mut PostgresFindings,
    ) {
        *ordinal += 1;
        let file = if crate::codebase::ts_source::matching_disable_directive(
            self.source,
            Some(call.line),
            RULE_ID,
        )
        .is_some()
        {
            offset.line = call.line as usize;
            self.rel.to_string()
        } else {
            relative_slash_path(self.root, origin)
        };
        findings.push(super::super::scan::finding(&file, &offset, *ordinal));
    }
}
