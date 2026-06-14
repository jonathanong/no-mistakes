use super::{extract::ExtractedSet, finding, Comparison};
use crate::codebase::rules::RuleFinding;
use globset::{Glob, GlobSetBuilder};

pub(super) fn compare(
    left: &ExtractedSet,
    right: &ExtractedSet,
    comparison: &Comparison,
    findings: &mut Vec<RuleFinding>,
) {
    match comparison.mode.as_str() {
        "" | "equal-set" => compare_equal_set(left, right, comparison, findings),
        "glob-coverage" => compare_glob_coverage(left, right, comparison, findings),
        "mention" => compare_mentions(left, right, comparison, findings),
        _ => {}
    }
}

fn compare_equal_set(
    left: &ExtractedSet,
    right: &ExtractedSet,
    comparison: &Comparison,
    findings: &mut Vec<RuleFinding>,
) {
    for value in left.values.difference(&right.values) {
        findings.push(finding(
            &right.file,
            comparison,
            format!(
                "{} contains `{}` but {} does not",
                comparison.left, value, comparison.right
            ),
            value,
        ));
    }
    for value in right.values.difference(&left.values) {
        findings.push(finding(
            &left.file,
            comparison,
            format!(
                "{} contains `{}` but {} does not",
                comparison.right, value, comparison.left
            ),
            value,
        ));
    }
}

fn compare_glob_coverage(
    left: &ExtractedSet,
    right: &ExtractedSet,
    comparison: &Comparison,
    findings: &mut Vec<RuleFinding>,
) {
    let mut builder = GlobSetBuilder::new();
    let mut has_invalid_glob = false;
    for pattern in &right.values {
        match Glob::new(pattern) {
            Ok(glob) => {
                builder.add(glob);
            }
            Err(error) => {
                has_invalid_glob = true;
                findings.push(finding(
                    &right.file,
                    comparison,
                    format!(
                        "{} contains invalid glob `{pattern}`: {error}",
                        comparison.right
                    ),
                    pattern,
                ));
            }
        }
    }
    if has_invalid_glob {
        return;
    }
    let globs = builder
        .build()
        .expect("glob set should build after every glob is validated");
    for value in &left.values {
        if !globs.is_match(value) {
            findings.push(finding(
                &right.file,
                comparison,
                format!(
                    "{} contains `{}` but no glob in {} covers it",
                    comparison.left, value, comparison.right
                ),
                value,
            ));
        }
    }
}

fn compare_mentions(
    left: &ExtractedSet,
    right: &ExtractedSet,
    comparison: &Comparison,
    findings: &mut Vec<RuleFinding>,
) {
    for value in left.values.difference(&right.values) {
        findings.push(finding(
            &right.file,
            comparison,
            format!(
                "{} contains `{}` but {} does not mention it",
                comparison.left, value, comparison.right
            ),
            value,
        ));
    }
}
