use super::{BannedFunctionCall, CompiledFunction, FunctionEntry};
use crate::codebase::postgres::{decoded_parts, SqlFunctionClause};
use crate::codebase::rules::postgres_sql_shape_policy::RULE_ID;
use anyhow::{bail, Result};
use std::collections::BTreeSet;

pub(in crate::codebase::rules::postgres_sql_shape_policy) fn compile(
    options: &BannedFunctionCall,
    enabled: bool,
) -> Result<Vec<CompiledFunction>> {
    if !enabled {
        return Ok(Vec::new());
    }
    if options.functions.is_empty() {
        bail!("{RULE_ID} option shapeOptions.bannedFunctionCall.functions: configure a nonempty function list");
    }
    let mut names = BTreeSet::new();
    options.functions.iter().map(|entry| {
        let (name, clauses, hint) = match entry {
            FunctionEntry::Name(name) => (name.as_str(), None, None),
            FunctionEntry::Scoped(options) => (options.name.as_str(), options.clauses.as_ref(), options.hint.as_ref()),
        };
        let parts = decoded_parts(name.trim());
        if name.trim().is_empty() || parts.iter().any(String::is_empty) {
            bail!("{RULE_ID} option shapeOptions.bannedFunctionCall.functions: empty function name");
        }
        if !names.insert(parts.clone()) {
            bail!("{RULE_ID} option shapeOptions.bannedFunctionCall.functions: duplicate function name {name}");
        }
        let clauses = clauses.map(|clauses| {
            if clauses.is_empty() { bail!("{RULE_ID} option shapeOptions.bannedFunctionCall.functions.clauses: must be nonempty"); }
            let mut seen = BTreeSet::new();
            clauses.iter().map(|clause| {
                let Some(value) = SqlFunctionClause::parse(clause) else { bail!("{RULE_ID} option shapeOptions.bannedFunctionCall.functions.clauses: unknown clause {clause}"); };
                if !seen.insert(value) { bail!("{RULE_ID} option shapeOptions.bannedFunctionCall.functions.clauses: duplicate clause {clause}"); }
                Ok(value)
            }).collect::<Result<Vec<_>>>()
        }).transpose()?;
        if hint.is_some_and(|hint| hint.trim().is_empty()) {
            bail!("{RULE_ID} option shapeOptions.bannedFunctionCall.functions.hint: must be nonempty");
        }
        Ok(CompiledFunction {name:parts, clauses, hint:hint.map(|hint| hint.trim().to_string())})
    }).collect()
}
