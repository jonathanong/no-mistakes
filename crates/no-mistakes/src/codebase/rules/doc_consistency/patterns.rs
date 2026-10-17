use super::{RuleFinding, RULE_ID};
use crate::codebase::rules::path_filter::GlobMatcher;
use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PatternSpec {
    files: Vec<String>,
    pattern: String,
    message: String,
}

pub(super) struct CompiledPattern {
    files: GlobMatcher,
    pattern: Regex,
    message: String,
}

pub(super) fn compile(specs: &[PatternSpec]) -> Result<Vec<CompiledPattern>> {
    specs
        .iter()
        .enumerate()
        .map(|(index, spec)| {
            let context = format!("{RULE_ID} bannedPatterns[{index}]");
            if spec.files.is_empty()
                || spec.files.iter().any(|file| file.trim().is_empty())
                || spec.pattern.is_empty()
                || spec.message.trim().is_empty()
            {
                bail!("{context} requires nonempty files, pattern, and message");
            }
            Ok(CompiledPattern {
                files: GlobMatcher::new(&spec.files, &context)?,
                pattern: Regex::new(&spec.pattern)
                    .with_context(|| format!("{context} contains an invalid regex"))?,
                message: spec.message.clone(),
            })
        })
        .collect()
}

impl CompiledPattern {
    pub(super) fn matches_path(&self, path: &str) -> bool {
        self.files.is_match(path)
    }
}

pub(super) fn findings(
    patterns: &[CompiledPattern],
    path: &str,
    content: &str,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for pattern in patterns.iter().filter(|pattern| pattern.matches_path(path)) {
        let (mut cursor, mut line, mut previous) = (0, 1, None);
        for matched in pattern.pattern.find_iter(content) {
            // Match starts are ordered; count each source byte at most once per pattern.
            line += content.as_bytes()[cursor..matched.start()]
                .iter()
                .filter(|byte| **byte == b'\n')
                .count();
            cursor = matched.start();
            if previous == Some(line) {
                continue;
            }
            previous = Some(line);
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: path.to_string(),
                line,
                message: pattern.message.clone(),
                import: None,
                target: None,
            });
        }
    }
    findings
}
