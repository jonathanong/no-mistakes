use super::{scan::finding, BannedShapes, RuleFinding, BANNED_FUNCTION_CALL, RULE_ID};
use crate::codebase::postgres::{decoded_parts, SqlStatementFileFacts};
use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct BannedFunctionCall {
    functions: Vec<String>,
}

pub(super) fn compile(options: &BannedFunctionCall, enabled: bool) -> Result<Vec<Vec<String>>> {
    if !enabled {
        return Ok(Vec::new());
    }
    if options.functions.is_empty() {
        bail!("{RULE_ID} option shapeOptions.bannedFunctionCall.functions: configure a nonempty function list");
    }
    options.functions.iter().map(|name| {
        let parts = decoded_parts(name.trim());
        if name.trim().is_empty() || parts.iter().any(String::is_empty) {
            bail!("{RULE_ID} option shapeOptions.bannedFunctionCall.functions: empty function name");
        }
        Ok(parts)
    }).collect()
}

pub(super) fn findings(
    file: &str,
    facts: &SqlStatementFileFacts,
    shapes: &BannedShapes,
    names: &[Vec<String>],
    line_at: impl Fn(usize) -> usize,
) -> Vec<RuleFinding> {
    if !shapes.banned_function_call {
        return Vec::new();
    }
    facts.function_calls.iter().filter(|call| names.iter().any(|name| name == &call.name_parts || name.len() == 1 && name.last() == call.name_parts.last())).map(|call| {
        let spelling = call.name_parts.iter().map(|part| format!("\"{}\"", part.replace('"', "\"\""))).collect::<Vec<_>>().join(".");
        finding(file, line_at(call.line), &format!("function call {spelling} is banned by this SQL shape policy; remove the call or replace it with an allowed operation"), BANNED_FUNCTION_CALL)
    }).collect()
}
