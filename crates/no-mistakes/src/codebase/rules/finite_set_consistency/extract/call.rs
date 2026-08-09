use super::*;

pub(super) fn extract_call_first_string_argument(
    root: &Path,
    path: &Path,
    spec: &SetSpec,
    facts: Option<&dyn TsFactLookup>,
    values: &mut BTreeSet<String>,
    issues: &mut Vec<ExtractionIssue>,
) {
    let file = relative_slash_path(root, path);
    if spec.target.is_empty() {
        issues.push(ExtractionIssue {
            file,
            message: format!(
                "finite set '{}' requires a non-empty target for kind '{}'",
                spec.name, TS_CALL_FIRST_STRING_ARGUMENT
            ),
            target: None,
        });
        return;
    }
    let Some(file_facts) = facts.and_then(|facts| facts.get_ts_facts(path)) else {
        issues.push(ExtractionIssue {
            file,
            message: format!(
                "finite set '{}' has no prepared TypeScript facts for its configured file",
                spec.name
            ),
            target: Some(spec.target.clone()),
        });
        return;
    };
    if let Some(error) = &file_facts.parse_error {
        issues.push(ExtractionIssue {
            file,
            message: format!(
                "finite set '{}' cannot extract calls because the configured file failed to parse: {error}",
                spec.name
            ),
            target: Some(spec.target.clone()),
        });
        return;
    }
    let mut matched = false;
    let mut has_non_static_argument = false;
    for call in &file_facts.function_calls {
        if call.callee != spec.target {
            continue;
        }
        matched = true;
        if let Some(value) = &call.static_arg {
            values.insert(value.clone());
        } else {
            has_non_static_argument = true;
        }
    }
    if !matched {
        issues.push(ExtractionIssue {
            file,
            message: format!(
                "finite set '{}' found no calls matching target '{}'",
                spec.name, spec.target
            ),
            target: Some(spec.target.clone()),
        });
        return;
    }
    if has_non_static_argument {
        issues.push(ExtractionIssue {
            file,
            message: format!(
                "finite set '{}' requires every '{}' call to have a static first string argument",
                spec.name, spec.target
            ),
            target: Some(spec.target.clone()),
        });
    }
}
