pub(in super::super::super) fn job_key_line(source: &str, job_id: &str, keys: &[&str]) -> usize {
    let lines: Vec<&str> = source.lines().collect();
    let Some(job_at) = job_offset(&lines, job_id) else {
        return 1;
    };
    let job_indent = indent(lines[job_at]);
    let end = lines
        .iter()
        .enumerate()
        .skip(job_at + 1)
        .find_map(|(offset, line)| {
            if line.trim().is_empty() {
                return None;
            }
            (indent(line) <= job_indent).then_some(offset + 1)
        })
        .unwrap_or(lines.len() + 1);
    let start = job_at + 1;
    keys.iter()
        .find_map(|key| key_between(source, start, end, key))
        .unwrap_or(start)
}

fn job_offset(lines: &[&str], job_id: &str) -> Option<usize> {
    let jobs_at = lines
        .iter()
        .position(|line| indent(line) == 0 && is_key(trim_comment(line), "jobs"))?;
    let parent = indent(lines[jobs_at]);
    let mut child = None;
    for (offset, line) in lines.iter().enumerate().skip(jobs_at + 1) {
        if line.trim().is_empty() {
            continue;
        }
        let indent = indent(line);
        if indent <= parent {
            break;
        }
        match child {
            None => child = Some(indent),
            Some(expected) if indent != expected => continue,
            Some(_) => {}
        }
        if is_key(trim_comment(line), job_id) {
            return Some(offset);
        }
    }
    None
}

fn key_between(source: &str, start: usize, end: usize, key: &str) -> Option<usize> {
    source
        .lines()
        .enumerate()
        .take(end.saturating_sub(1))
        .skip(start.saturating_sub(1))
        .find_map(|(offset, line)| {
            let trimmed = trim_comment(line);
            let body = trimmed.strip_prefix("- ").unwrap_or(trimmed);
            is_key(body, key).then_some(offset + 1)
        })
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
