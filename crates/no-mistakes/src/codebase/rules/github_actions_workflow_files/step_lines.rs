pub(in super::super) fn first_key_line(
    source: &str,
    job_id: &str,
    index: usize,
    keys: &[&str],
) -> usize {
    let Some(start) = step_start_line(source, job_id, index) else {
        return 1;
    };
    let end = next_step_end(source, start);
    keys.iter()
        .find_map(|key| key_between(source, start, end, key))
        .unwrap_or(start)
}

pub(in super::super) fn job_key_line(source: &str, job_id: &str, keys: &[&str]) -> usize {
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

fn step_start_line(source: &str, job_id: &str, index: usize) -> Option<usize> {
    let lines: Vec<&str> = source.lines().collect();
    let from = if job_id == "(composite)" {
        0
    } else {
        job_offset(&lines, job_id)?
    };
    let steps_at = lines
        .iter()
        .enumerate()
        .skip(from)
        .find_map(|(offset, line)| is_key(trim_comment(line), "steps").then_some(offset))?;
    let steps_indent = indent(lines[steps_at]);
    let mut step_indent = None;
    let mut seen = 0usize;
    for (offset, line) in lines.iter().enumerate().skip(steps_at + 1) {
        if line.trim().is_empty() {
            continue;
        }
        let indent = indent(line);
        if indent <= steps_indent {
            break;
        }
        if !trim_comment(line).starts_with("- ") {
            continue;
        }
        match step_indent {
            None => step_indent = Some(indent),
            Some(expected) if indent != expected => continue,
            Some(_) => {}
        }
        if seen == index {
            return Some(offset + 1);
        }
        seen += 1;
    }
    None
}

fn next_step_end(source: &str, start: usize) -> usize {
    let lines: Vec<&str> = source.lines().collect();
    let start_idx = start.saturating_sub(1);
    let start_indent = indent(lines.get(start_idx).copied().unwrap_or(""));
    lines
        .iter()
        .enumerate()
        .skip(start_idx + 1)
        .find_map(|(offset, line)| {
            if line.trim().is_empty() {
                return None;
            }
            let indent = indent(line);
            let trimmed = trim_comment(line);
            ((trimmed.starts_with("- ") && indent <= start_indent) || indent < start_indent)
                .then_some(offset + 1)
        })
        .unwrap_or(lines.len() + 1)
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
