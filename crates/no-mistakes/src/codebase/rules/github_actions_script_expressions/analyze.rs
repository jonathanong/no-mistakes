use super::hit::Hit;
use super::line_scan::hits_from_lines;
use super::value_walk::{hits_from_value, is_composite};
use super::{RuleFinding, RULE_ID};
use regex::Regex;
use serde_yaml::Value;

pub(super) fn gha_kind(rel: &str) -> Option<bool> {
    let yaml = rel.ends_with(".yml") || rel.ends_with(".yaml");
    if !yaml {
        return None;
    }
    if rel.starts_with(".github/workflows/") {
        return Some(false);
    }
    if rel.starts_with(".github/actions/")
        && (rel.ends_with("/action.yml") || rel.ends_with("/action.yaml"))
    {
        return Some(true);
    }
    None
}

pub(super) fn check_source(rel: &str, source: &str, patterns: &[Regex]) -> Vec<RuleFinding> {
    let source = source.trim_start_matches('\u{feff}');
    let value = match serde_yaml::from_str::<Value>(source) {
        Ok(value) => value,
        Err(err) => {
            return vec![finding(
                rel,
                err.location().map_or(1, |location| location.line()),
                format!(
                    "{rel}: invalid YAML ({err}). This rule cannot judge script expressions in a file it cannot parse."
                ),
                None,
            )];
        }
    };
    let action = gha_kind(rel).unwrap_or(false);
    let composite = action && is_composite(&value);
    let yaml_hits = hits_from_value(&value, action);
    let line_hits = hits_from_lines(source, action, composite);
    if !aligned(&yaml_hits, &line_hits) {
        return vec![finding(
            rel,
            1,
            format!(
                "{rel}: script expressions could not be aligned to source locations. Rewrite anchors, aliases, and flow-style steps into ordinary block-style `run` or `with.script` bodies so this rule can judge them."
            ),
            None,
        )];
    }
    line_hits
        .into_iter()
        .filter(|hit| !allows(patterns, &hit.expr))
        .map(|hit| {
            let target = hit.expr.clone();
            finding(
                rel,
                hit.line,
                script_message(rel, hit.line, &hit),
                Some(target),
            )
        })
        .collect()
}

fn aligned(yaml_hits: &[Hit], line_hits: &[Hit]) -> bool {
    yaml_hits.len() == line_hits.len()
        && yaml_hits
            .iter()
            .zip(line_hits)
            .all(|(yaml, line)| yaml.script == line.script && yaml.expr == line.expr)
}

fn allows(patterns: &[Regex], expr: &str) -> bool {
    let expr = expr.trim();
    patterns.iter().any(|pattern| {
        pattern
            .find(expr)
            .is_some_and(|matched| matched.as_str() == expr)
    })
}

fn script_message(rel: &str, line: usize, hit: &Hit) -> String {
    if hit.script {
        format!(
            "{rel}:{line}: `${{{{ {} }}}}` in an `actions/github-script` `script` is expanded before JavaScript parses the script. Pass the value through `env` and read `process.env`.",
            hit.expr.trim()
        )
    } else {
        format!(
            "{rel}:{line}: `${{{{ {} }}}}` in a `run` script is expanded before the shell parses the script. Pass the value through `env` and read it as data (`env: VALUE: ${{{{ {} }}}}`, then `run: echo \"$VALUE\"`).",
            hit.expr.trim(),
            hit.expr.trim()
        )
    }
}

fn finding(rel: &str, line: usize, message: String, target: Option<String>) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line,
        message,
        import: None,
        target,
    }
}
