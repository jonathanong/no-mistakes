//! Per-file helper summaries, computed once with the rest of the walk.

use super::super::super::super::helpers::returns;
use super::super::super::super::helpers::split::compose;
use super::super::super::super::helpers::template::{self, TemplateHelper};
use super::super::super::super::helpers::{BuilderHelper, HelperSummary};
use super::provenance::RecoveredSql;
use super::reassigned::ReassignedNames;
use super::{LocalFunctions, Resolvable};
use crate::fx::FxHashMap;
use oxc_ast::ast::Program;
use std::collections::HashMap;

/// What a helper body may call: other same-file helpers and template helpers.
pub(super) struct Callables<'r, 'a> {
    pub(super) raw: &'r HashMap<&'a str, Resolvable<'a>>,
    pub(super) templates: &'r FxHashMap<String, TemplateHelper>,
}

/// Literal-identifier templates declared at the top level and never reassigned.
pub(super) fn template_helpers(
    program: &Program<'_>,
    source: &str,
    reassigned: &ReassignedNames<'_>,
) -> FxHashMap<String, TemplateHelper> {
    returns::collect(program)
        .into_iter()
        .filter(|(name, _, _)| !reassigned.contains(name))
        .filter_map(|(name, params, returned)| {
            Some((
                name.to_string(),
                template::summarize(params, returned, source)?,
            ))
        })
        .collect()
}

impl LocalFunctions {
    /// Every helper in this file that an importer could call, by local name.
    pub(crate) fn exported_helpers(&self) -> FxHashMap<String, HelperSummary> {
        let mut summaries = FxHashMap::default();
        for (name, template) in &self.templates {
            summaries.insert(name.clone(), HelperSummary::Template(template.clone()));
        }
        for (name, sql) in &self.resolved {
            summaries.insert(name.clone(), HelperSummary::Static(sql.clone()));
        }
        for (name, builder) in &self.parameter_builders {
            let suffix = compose(builder.fragments.iter().map(|fragment| RecoveredSql {
                sql: fragment.sql.clone(),
                line: fragment.line,
                positions: fragment.positions.clone(),
                origins: fragment.origins.clone(),
            }));
            summaries.insert(
                name.clone(),
                HelperSummary::Builder(BuilderHelper {
                    parameter_index: builder.parameter_index,
                    parameter_count: builder.parameter_count,
                    suffix: suffix.unwrap_or_else(|| RecoveredSql {
                        sql: String::new(),
                        line: 1,
                        positions: Vec::new(),
                        origins: Vec::new(),
                    }),
                }),
            );
        }
        summaries
    }
}
