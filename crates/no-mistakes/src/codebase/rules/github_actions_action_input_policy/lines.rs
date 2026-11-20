// A direct `with:` key is the input. A key nested under `<<` is not.
pub(super) fn input_line(source: &str, job_id: &str, index: usize, key: &str) -> usize {
    let with_line = step_line(source, job_id, index, &["with"]);
    if let Some(line) = direct_with_input(source, with_line, key) {
        return line;
    }
    step_line(source, job_id, index, &["uses"])
}

pub(super) fn job_input_line(source: &str, job_id: &str, key: &str) -> usize {
    if let Some(with_line) = job_child_line(source, job_id, "with") {
        if let Some(line) = direct_with_input(source, with_line, key) {
            return line;
        }
    }
    if let Some(line) = job_child_line(source, job_id, "uses") {
        return line;
    }
    super::super::github_actions_workflow_files::step_lines::job_key_line(source, job_id, &["uses"])
}

fn step_line(source: &str, job_id: &str, index: usize, keys: &[&str]) -> usize {
    super::super::github_actions_workflow_files::step_lines::first_key_line(
        source, job_id, index, keys,
    )
}

fn direct_with_input(source: &str, with_line: usize, key: &str) -> Option<usize> {
    let lines: Vec<&str> = source.lines().collect();
    direct_child_key(&lines, with_line, key)
}

fn job_child_line(source: &str, job_id: &str, child_key: &str) -> Option<usize> {
    let lines: Vec<&str> = source.lines().collect();
    let jobs_at = lines
        .iter()
        .position(|line| indent(line) == 0 && is_key(trim_comment(line), "jobs"))?;
    let jobs_indent = indent(lines[jobs_at]);
    let mut job_indent = None;
    let mut job_at = None;
    for (offset, line) in lines.iter().enumerate().skip(jobs_at + 1) {
        if line.trim().is_empty() {
            continue;
        }
        let current = indent(line);
        if current <= jobs_indent {
            break;
        }
        match job_indent {
            None => job_indent = Some(current),
            Some(expected) if current != expected => continue,
            Some(_) => {}
        }
        if is_key(trim_comment(line), job_id) {
            job_at = Some(offset);
            break;
        }
    }
    let job_at = job_at?;
    let own_indent = indent(lines[job_at]);
    let mut child_indent = None;
    for (offset, line) in lines.iter().enumerate().skip(job_at + 1) {
        if line.trim().is_empty() {
            continue;
        }
        let current = indent(line);
        if current <= own_indent {
            break;
        }
        match child_indent {
            None => child_indent = Some(current),
            Some(expected) if current != expected => continue,
            Some(_) => {}
        }
        if is_key(trim_comment(line), child_key) {
            return Some(offset + 1);
        }
    }
    None
}

fn direct_child_key(lines: &[&str], parent_line: usize, key: &str) -> Option<usize> {
    let parent = lines.get(parent_line.saturating_sub(1))?;
    if !is_key(trim_comment(parent), "with") {
        return None;
    }
    let parent_indent = indent(parent);
    let mut child_indent = None;
    for (offset, line) in lines.iter().enumerate().skip(parent_line) {
        if line.trim().is_empty() {
            continue;
        }
        let current = indent(line);
        if current <= parent_indent {
            break;
        }
        match child_indent {
            None => child_indent = Some(current),
            Some(expected) if current != expected => continue,
            Some(_) => {}
        }
        if is_key(trim_comment(line), key) {
            return Some(offset + 1);
        }
    }
    None
}

fn is_key(trimmed: &str, key: &str) -> bool {
    let prefix = format!("{key}:");
    trimmed == prefix || trimmed.starts_with(&prefix)
}

fn trim_comment(line: &str) -> &str {
    line.split('#').next().unwrap_or(line).trim()
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}
