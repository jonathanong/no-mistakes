use super::env_read;
use super::pairs::PairContext;
use crate::codebase::rules::github_actions_workflow_files::job_lines::document::Document;
use crate::codebase::rules::github_actions_workflow_files::shell_words::Word;

pub(super) fn extract_flag_pair<'a>(
    command: &'a [Word],
    base_flag: &str,
    head_flag: &str,
) -> Option<(Option<&'a str>, Option<&'a str>)> {
    let words: Vec<&str> = command
        .iter()
        .flat_map(|word| word.text.split_whitespace())
        .collect();
    let mut base_seen = false;
    let mut head_seen = false;
    let mut base_val: Option<&str> = None;
    let mut head_val: Option<&str> = None;
    let mut index = 0;
    while index < words.len() {
        let word = words[index].trim_end_matches([')', '`']);
        if word == "--" {
            break;
        }
        if let Some((val, step)) = match_flag(&words, index, base_flag) {
            base_val = if base_seen && base_val != val {
                None
            } else {
                val
            };
            base_seen = true;
            index += step;
            continue;
        }
        if let Some((val, step)) = match_flag(&words, index, head_flag) {
            head_val = if head_seen && head_val != val {
                None
            } else {
                val
            };
            head_seen = true;
            index += step;
            continue;
        }
        index += 1;
    }
    (base_seen && head_seen).then_some((base_val, head_val))
}

fn match_flag<'a>(words: &[&'a str], index: usize, flag: &str) -> Option<(Option<&'a str>, usize)> {
    let word = words[index].trim_end_matches([')', '`']);
    if word == flag {
        let next = words
            .get(index + 1)
            .map(|w| w.trim_end_matches([')', '`']));
        return Some((next, if next.is_some() { 2 } else { 1 }));
    }
    let rest = word.strip_prefix(flag)?.strip_prefix('=')?;
    Some(((!rest.is_empty()).then_some(rest), 1))
}

pub(super) fn env_pair_line(ctx: &PairContext<'_>, head: &str, base: &str) -> usize {
    let doc = ctx.document.get_or_init(|| Document::parse(ctx.source));
    let Some(doc) = doc.as_ref() else {
        return ctx.located.map_or(1, |run| run.item_line);
    };
    if let Some(job) = ctx.job {
        let step_idx = ctx.step_index.map_or(usize::MAX, |i| i.saturating_sub(1));
        if let Some(line) = env_read::line(doc, job, step_idx, head)
            .or_else(|| env_read::line(doc, job, step_idx, base))
        {
            return line;
        }
    }
    let Some(root) = doc.roots().first().copied() else {
        return 1;
    };
    doc.field(root, "env")
        .and_then(|env| doc.field_line(env, head).or_else(|| doc.field_line(env, base)))
        .unwrap_or(1)
}

pub(super) fn with_pair_line(ctx: &PairContext<'_>, head: &str, base: &str) -> usize {
    let doc = ctx.document.get_or_init(|| Document::parse(ctx.source));
    let Some(doc) = doc.as_ref() else {
        return 1;
    };
    let Some(root) = doc.roots().first().copied() else {
        return 1;
    };
    let Some(job_name) = ctx.job else {
        return 1;
    };
    let Some(job) = doc
        .field(root, "jobs")
        .and_then(|jobs| doc.field(jobs, job_name))
    else {
        return 1;
    };
    let owner = match ctx.step_index {
        Some(index) => doc
            .field(job, "steps")
            .and_then(|steps| doc.sequence(steps))
            .and_then(|steps| steps.get(index.saturating_sub(1)))
            .copied()
            .unwrap_or(job),
        None => job,
    };
    doc.field(owner, "with")
        .and_then(|with| doc.field_line(with, head).or_else(|| doc.field_line(with, base)))
        .unwrap_or(1)
}
