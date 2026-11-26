use super::env_read;
use super::pairs::{PairContext, PairOutput};
use super::verdicts::Verdict;
use super::RULE_ID;
use crate::codebase::rules::github_actions_workflow_files::job_lines::document::Document;
use crate::codebase::rules::github_actions_workflow_files::shell_words::Word;
use crate::codebase::rules::RuleFinding;

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

fn owner_node(doc: &Document, ctx: &PairContext<'_>) -> Option<usize> {
    let root = *doc.roots().first()?;
    let Some(job_name) = ctx.job else {
        return Some(root);
    };
    let job = doc.field(doc.field(root, "jobs")?, job_name)?;
    Some(
        ctx.step_index
            .and_then(|index| {
                doc.sequence(doc.field(job, "steps")?)?
                    .get(index.saturating_sub(1))
                    .copied()
            })
            .unwrap_or(job),
    )
}

pub(super) fn field_line(ctx: &PairContext<'_>, field: &str) -> usize {
    ctx.document
        .get_or_init(|| Document::parse(ctx.source))
        .as_ref()
        .and_then(|doc| doc.field_line(owner_node(doc, ctx)?, field))
        .unwrap_or(1)
}

pub(super) fn env_pair_line(ctx: &PairContext<'_>, head: &str, base: &str) -> usize {
    ctx.document
        .get_or_init(|| Document::parse(ctx.source))
        .as_ref()
        .and_then(|doc| {
            if let Some(job) = ctx.job {
                let step_idx = ctx.step_index.map_or(usize::MAX, |i| i.saturating_sub(1));
                env_read::line(doc, job, step_idx, head)
                    .or_else(|| env_read::line(doc, job, step_idx, base))
            } else {
                let root = *doc.roots().first()?;
                let env = doc.field(root, "env")?;
                doc.field_line(env, head)
                    .or_else(|| doc.field_line(env, base))
            }
        })
        .unwrap_or(1)
}

pub(super) fn with_pair_line(ctx: &PairContext<'_>, head: &str, base: &str) -> usize {
    ctx.document
        .get_or_init(|| Document::parse(ctx.source))
        .as_ref()
        .and_then(|doc| {
            let with = doc.field(owner_node(doc, ctx)?, "with")?;
            doc.field_line(with, head)
                .or_else(|| doc.field_line(with, base))
        })
        .unwrap_or(1)
}

pub(super) fn emit(
    ctx: &PairContext<'_>,
    line: usize,
    label: &str,
    verdict: Verdict,
    out: &mut PairOutput<'_>,
) {
    let where_text = match (ctx.job, ctx.step_index) {
        (Some(job), Some(step)) => format!("job `{job}` step {step}"),
        (Some(job), None) => format!("job `{job}`"),
        (None, _) => "workflow".to_string(),
    };
    let (bucket, target, reason) = match verdict {
        Verdict::Consistent => return,
        Verdict::Mismatch(reason) => (&mut *out.findings, ctx.job.map(str::to_owned), reason),
        Verdict::Unanalyzable(reason) => {
            (&mut *out.undecidable, Some("unanalyzable".to_string()), reason)
        }
    };
    bucket.push(RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: ctx.rel.to_string(),
        line,
        message: format!("{}:{line}: {where_text} ({label}) {reason}.", ctx.rel),
        import: None,
        target,
    });
}
