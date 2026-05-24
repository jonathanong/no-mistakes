use regex::Regex;

pub(crate) fn is_managed_runner(v: &str) -> bool {
    v.starts_with("ubuntu-") || v.starts_with("macos-") || v.starts_with("windows-")
}

pub(crate) fn is_managed_runner_only(content: &str) -> bool {
    let runs_on_re = Regex::new(r"(?m)^\s*runs-on:\s*(.+?)\s*$").expect("runs-on regex");
    let empty_runs_on_re = Regex::new(r"(?m)^\s*runs-on:\s*$").expect("empty runs-on regex");
    let mut values = Vec::new();
    let mut in_runs_on_list = false;
    for line in content.lines() {
        if let Some(cap) = runs_on_re.captures(line) {
            values.extend(parse_runs_on_values(cap.get(1).map_or("", |m| m.as_str())));
            in_runs_on_list = false;
            continue;
        }
        if empty_runs_on_re.is_match(line) {
            in_runs_on_list = true;
            continue;
        }
        if in_runs_on_list {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some(item) = trimmed.strip_prefix("- ") {
                values.extend(parse_runs_on_values(item));
                continue;
            }
            if let Some((key, value)) = trimmed.split_once(':') {
                match key.trim() {
                    "group" => continue,
                    "labels" => {
                        values.extend(parse_runs_on_values(value));
                        continue;
                    }
                    _ => {}
                }
            }
            in_runs_on_list = false;
        }
    }
    if values.is_empty() {
        return false;
    }
    values.iter().all(|runner| is_managed_runner(runner))
}

pub(crate) fn parse_runs_on_values(raw: &str) -> Vec<String> {
    let value = raw
        .split_once('#')
        .map_or(raw, |(before_comment, _)| before_comment)
        .trim()
        .trim_matches(|c| matches!(c, '[' | ']'));
    value
        .split(',')
        .map(|part| {
            part.trim()
                .trim_start_matches("- ")
                .trim()
                .trim_matches(|c| matches!(c, '\'' | '"'))
        })
        .filter(|part| !part.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
