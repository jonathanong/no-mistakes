use super::{Body, Hit, RuleFinding, StepRun, RULE_ID};

pub(super) fn workflow_finding(
    rel: &str,
    body: &Body<'_>,
    place: Option<&StepRun>,
    hit: &Hit,
    strict: bool,
) -> RuleFinding {
    let line = place
        .and_then(|place| place.lines.get(hit.line.saturating_sub(1)))
        .map(|(line, _)| *line)
        .or_else(|| place.and_then(|place| place.lines.first().map(|(line, _)| *line)))
        .unwrap_or(1);
    let location = format!("job `{}` step {}", body.target, body.step);
    finding(rel, line, &location, &body.target, hit, strict)
}

pub(super) fn script_finding(rel: &str, hit: &Hit, strict: bool) -> RuleFinding {
    finding(rel, hit.line, "shell script", "shell", hit, strict)
}

fn finding(
    rel: &str,
    line: usize,
    location: &str,
    target: &str,
    hit: &Hit,
    strict: bool,
) -> RuleFinding {
    let url = &hit.url;
    let command = &hit.command;
    let message = if hit.indirect_command {
        format!(
            "downloader command is unanalyzable at effort low; call `{command}` directly with literal HTTP(S) URL arguments instead of a command alias."
        )
    } else if hit.unanalyzable {
        format!(
            "download URL or argument shape `{url}` is unanalyzable at effort low; inline a literal HTTP(S) URL in the `{command}` argument, use literal option names, and quote output or header values instead of allowing assignments, expansions, or command substitutions to change argument positions."
        )
    } else {
        let when = if strict {
            "before the next unrelated download or the end of this script"
        } else if location == "shell script" {
            "in this script"
        } else {
            "in this step"
        };
        format!(
            "downloads `{url}` with `{command}` and does not verify the bytes {when}. An http(s) download can change between runs. Verify it here with `sha256sum -c` or `shasum -a 256 -c`, download a checksum file, use a trusted host, or add `# integrity-check: skip reason=<slug>`."
        )
    };
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line,
        message: format!("{rel}:{line}: {location} {message}"),
        import: None,
        target: Some(target.to_string()),
    }
}
