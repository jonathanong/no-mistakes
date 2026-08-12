use super::super::contracts::valid_identifier;

pub(super) fn context_property_name<'a>(operand: &'a str, context: &str) -> Option<&'a str> {
    let operand = operand.trim();
    let remainder = operand
        .get(..context.len())
        .filter(|prefix| prefix.eq_ignore_ascii_case(context))
        .and_then(|_| operand.get(context.len()..))?;
    if let Some(name) = remainder.trim_start().strip_prefix('.') {
        let name = name.trim();
        return valid_identifier(name).then_some(name);
    }
    let bracketed = remainder.trim_start().strip_prefix('[')?.trim_start();
    let quote = bracketed.chars().next()?;
    if quote != '\'' {
        return None;
    }
    let name = bracketed.strip_prefix(quote)?;
    let (name, suffix) = name.split_once(quote)?;
    (suffix.trim() == "]" && valid_identifier(name)).then_some(name)
}

pub(super) fn context_property_segment(remainder: &str) -> Option<(&str, &str)> {
    let remainder = remainder.trim_start();
    if let Some(remainder) = remainder.strip_prefix('.') {
        let remainder = remainder.trim_start();
        let end = remainder
            .find(|character: char| {
                character == '.' || character == '[' || character.is_whitespace()
            })
            .unwrap_or(remainder.len());
        let name = &remainder[..end];
        return valid_identifier(name).then_some((name, &remainder[end..]));
    }
    let quoted = remainder
        .strip_prefix('[')?
        .trim_start()
        .strip_prefix('\'')?;
    let (name, remainder) = quoted.split_once('\'')?;
    let remainder = remainder.trim_start().strip_prefix(']')?;
    valid_identifier(name).then_some((name, remainder))
}

pub(crate) fn context_output_name<'a>(
    operand: &'a str,
    context: &str,
) -> Option<(&'a str, &'a str)> {
    let operand = operand.trim();
    let remainder = operand
        .get(..context.len())
        .filter(|prefix| prefix.eq_ignore_ascii_case(context))
        .and_then(|_| operand.get(context.len()..))?;
    let (job, remainder) = context_property_segment(remainder)?;
    let remainder = github_property_segment(remainder, "outputs")?;
    let (output, remainder) = context_property_segment(remainder)?;
    remainder.trim().is_empty().then_some((job, output))
}

pub(in super::super) fn github_event_name(operand: &str) -> bool {
    github_event_property(operand, &["event_name"])
}

pub(in super::super) fn github_event_action(operand: &str) -> bool {
    github_event_property(operand, &["event", "action"])
}

pub(in super::super) fn github_pull_request_merged(operand: &str) -> bool {
    github_event_property(operand, &["event", "pull_request", "merged"])
}

pub(in super::super) fn github_ref(operand: &str) -> bool {
    github_event_property(operand, &["ref"])
}

pub(in super::super) fn github_ref_name(operand: &str) -> bool {
    github_event_property(operand, &["ref_name"])
}

pub(in super::super) fn github_ref_type(operand: &str) -> bool {
    github_event_property(operand, &["ref_type"])
}

pub(in super::super) fn github_base_ref(operand: &str) -> bool {
    github_event_property(operand, &["base_ref"])
        || github_event_property(operand, &["event", "pull_request", "base", "ref"])
}

pub(in super::super) fn github_head_ref(operand: &str) -> bool {
    github_event_property(operand, &["head_ref"])
}

fn github_event_property(operand: &str, properties: &[&str]) -> bool {
    let operand = operand.trim();
    let Some(remainder) = operand
        .get(.."github".len())
        .filter(|prefix| prefix.eq_ignore_ascii_case("github"))
        .and_then(|_| operand.get("github".len()..))
    else {
        return false;
    };
    let Some(remainder) = properties
        .iter()
        .try_fold(remainder, |remainder, property| {
            github_property_segment(remainder, property)
        })
    else {
        return false;
    };
    remainder.trim().is_empty()
}

pub(super) fn github_property_segment<'a>(remainder: &'a str, expected: &str) -> Option<&'a str> {
    let remainder = remainder.trim_start();
    if let Some(remainder) = remainder.strip_prefix('.') {
        let remainder = remainder.trim_start();
        let property = remainder.get(..expected.len())?;
        return property
            .eq_ignore_ascii_case(expected)
            .then_some(&remainder[expected.len()..]);
    }
    let remainder = remainder.strip_prefix('[')?.trim_start();
    let quoted = remainder.strip_prefix('\'')?;
    let (property, remainder) = quoted.split_once('\'')?;
    property
        .eq_ignore_ascii_case(expected)
        .then_some(remainder.trim_start().strip_prefix(']')?)
}
