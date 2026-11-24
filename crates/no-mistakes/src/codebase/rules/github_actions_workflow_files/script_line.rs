use crate::codebase::rules::github_actions_script_expressions::StepRun;

/// Map a 1-based line inside a `run` script to the YAML line that holds it.
pub(in crate::codebase::rules) fn yaml_line(
    located: Option<&StepRun>,
    script: &str,
    script_line: usize,
    token: &str,
) -> usize {
    let Some(located) = located.filter(|item| !item.lines.is_empty()) else {
        return 1;
    };
    if script.lines().count() == located.lines.len() {
        return located
            .lines
            .get(script_line.saturating_sub(1))
            .map(|(line, _)| *line)
            .unwrap_or(located.lines[0].0);
    }
    located
        .lines
        .iter()
        .find(|(_, text)| text.contains(token))
        .map(|(line, _)| *line)
        .unwrap_or(located.lines[0].0)
}
