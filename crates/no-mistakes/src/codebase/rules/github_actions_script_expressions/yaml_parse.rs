use super::yaml_text::{is_block_header, leading_indent, strip_unquoted_comment};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Jobs,
    Job,
    Steps,
    Step,
    Runs,
    With,
    Env,
    Other,
    /// Block scalar opened by a key at `Frame::indent`. `bool` is literal `|`/`>`
    /// (hash lines are script text) versus a plain scalar (hash lines are YAML comments).
    Block(BlockRole, bool),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum BlockRole {
    Run,
    Script,
    Other,
}

pub(super) struct Frame {
    pub(super) indent: usize,
    pub(super) kind: Kind,
}

pub(super) struct Parsed {
    pub(super) scope: usize,
    pub(super) key_indent: usize,
    pub(super) is_item: bool,
    pub(super) key: Option<String>,
    pub(super) value: String,
    pub(super) block: bool,
    pub(super) nested: bool,
}

pub(super) fn script_role(key: &str, parent: Option<Kind>) -> Option<BlockRole> {
    if key == "run" && parent == Some(Kind::Step) {
        Some(BlockRole::Run)
    } else if key == "script" && parent == Some(Kind::With) {
        Some(BlockRole::Script)
    } else {
        None
    }
}

pub(super) fn classify_key(key: &str, parent: Option<Kind>, action: bool, composite: bool) -> Kind {
    if key == "jobs" && !action && parent.is_none() {
        return Kind::Jobs;
    }
    if key == "runs" && action && parent.is_none() {
        return Kind::Runs;
    }
    if parent == Some(Kind::Jobs) {
        return Kind::Job;
    }
    if key == "steps" && parent == Some(Kind::Job) {
        return Kind::Steps;
    }
    if key == "steps" && parent == Some(Kind::Runs) && composite {
        return Kind::Steps;
    }
    if key == "env" {
        return Kind::Env;
    }
    if key == "with" && parent == Some(Kind::Step) {
        return Kind::With;
    }
    Kind::Other
}

pub(super) fn next_is_child_key(lines: &[&str], index: usize, parent_indent: usize) -> bool {
    lines
        .iter()
        .skip(index + 1)
        .find(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .is_some_and(|line| {
            leading_indent(line) > parent_indent
                && parse_line(line).is_some_and(|parsed| parsed.key.is_some())
        })
}

pub(super) fn parse_line(line: &str) -> Option<Parsed> {
    let indent = leading_indent(line);
    let trimmed = line[indent..].trim_end();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "---" || trimmed == "..." {
        return None;
    }
    let (is_item, scope, content, key_indent) = if let Some(after_dash) = trimmed.strip_prefix("- ")
    {
        let content = after_dash.trim_start();
        let key_indent = indent + 2 + (after_dash.len() - content.len());
        (true, indent, content, key_indent)
    } else if trimmed == "-" {
        return Some(empty_parsed(indent, indent, true));
    } else {
        (false, indent, trimmed, indent)
    };
    if content.is_empty() || content.starts_with('#') {
        return Some(empty_parsed(scope, key_indent, is_item));
    }
    let (key, value_raw) = content.split_once(':')?;
    let key = clean_key(key)?;
    let value_raw = value_raw.trim();
    let value = strip_unquoted_comment(value_raw).trim();
    let block = is_block_header(value);
    let nested = !block && value.is_empty();
    Some(Parsed {
        scope,
        key_indent,
        is_item,
        key: Some(key),
        value: if block {
            String::new()
        } else {
            value.to_string()
        },
        block,
        nested,
    })
}

fn empty_parsed(scope: usize, key_indent: usize, is_item: bool) -> Parsed {
    Parsed {
        scope,
        key_indent,
        is_item,
        key: None,
        value: String::new(),
        block: false,
        nested: false,
    }
}

fn clean_key(key: &str) -> Option<String> {
    let key = key.trim().trim_matches(|ch| ch == '"' || ch == '\'');
    if key.is_empty()
        || key.contains([' ', '{', '[', ']', '}', ','])
        || key.starts_with('!')
        || key.starts_with('&')
        || key.starts_with('*')
        || key.starts_with('?')
    {
        return None;
    }
    Some(key.to_string())
}
