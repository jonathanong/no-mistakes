use super::{Body, Hit, RuleFinding, StepRun, RULE_ID};

pub(super) fn finding(rel: &str, body: &Body, place: Option<&StepRun>, hit: Hit) -> RuleFinding {
    let line = place
        .and_then(|place| place.lines.get(hit.line.saturating_sub(1)))
        .map(|(line, _)| *line)
        .or_else(|| place.and_then(|place| place.lines.first().map(|(line, _)| *line)))
        .unwrap_or(1);
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line,
        message: if hit.indirect_command {
            format!(
                "{rel}:{line}: job `{}` step {} downloader command is unanalyzable at effort low; call `{}` directly with literal HTTP(S) URL arguments instead of a command alias.",
                body.target, body.step, hit.command,
            )
        } else if hit.unanalyzable {
            format!(
                "{rel}:{line}: job `{}` step {} download URL or argument shape `{}` is unanalyzable at effort low; inline a literal HTTP(S) URL in the `{}` argument and use literal option names instead of assignments, expansions, or command substitutions.",
                body.target, body.step, hit.url, hit.command,
            )
        } else {
            format!(
                "{rel}:{line}: job `{}` step {} downloads `{url}` with `{command}` and does not \
             verify the bytes in this step. An http(s) download can change between runs. Verify \
             it here with `sha256sum -c` or `shasum -a 256 -c`, download a checksum file, use a \
             trusted host, or add `# integrity-check: skip reason=<slug>`.",
                body.target,
                body.step,
                command = hit.command,
                url = hit.url
            )
        },
        import: None,
        target: Some(body.target.clone()),
    }
}
