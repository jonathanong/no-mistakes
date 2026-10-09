use super::{scan::finding, BannedShapes, RuleFinding, BANNED_FUNCTION_CALL};
use crate::codebase::postgres::{SqlFunctionClause, SqlStatementFileFacts};
use serde::Deserialize;

mod compile;
pub(super) use compile::compile;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct BannedFunctionCall {
    functions: Vec<FunctionEntry>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum FunctionEntry {
    Name(String),
    Scoped(FunctionOptions),
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct FunctionOptions {
    name: String,
    clauses: Option<Vec<String>>,
    hint: Option<String>,
}

pub(super) struct CompiledFunction {
    name: Vec<String>,
    clauses: Option<Vec<SqlFunctionClause>>,
    hint: Option<String>,
}

pub(super) fn findings(
    file: &str,
    facts: &SqlStatementFileFacts,
    shapes: &BannedShapes,
    names: &[CompiledFunction],
    line_at: impl Fn(usize) -> usize,
) -> Vec<RuleFinding> {
    if !shapes.banned_function_call {
        return Vec::new();
    }
    facts.function_calls.iter().filter_map(|call| {
        let entry = names.iter().find(|entry| {
            (entry.name == call.name_parts || entry.name.len() == 1 && entry.name.last() == call.name_parts.last())
                && entry.clauses.as_ref().is_none_or(|clauses| call.clause.is_some_and(|clause| clauses.contains(&clause)))
        })?;
        let text = if entry.clauses.is_some() {
            format!("{}() is banned in {}", call.name_parts.join("."), call.clause.expect("matched a configured clause").label())
        } else {
            let spelling = call.name_parts.iter().map(|part| format!("\"{}\"", part.replace('"', "\"\""))).collect::<Vec<_>>().join(".");
            format!("function call {spelling} is banned by this SQL shape policy; remove the call or replace it with an allowed operation")
        };
        let text = entry.hint.as_ref().map_or_else(|| text.clone(), |hint| format!("{text}; {hint}"));
        Some(finding(file, line_at(call.line), &text, BANNED_FUNCTION_CALL))
    }).collect()
}
