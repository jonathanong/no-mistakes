const TAG: &str = "lock-reason:";
const PREVENTS: &str = "prevents:";
const WRITER: &str = "writer:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReasonStatus {
    Absent,
    MissingBoth,
    MissingPrevents,
    MissingWriter,
    Valid,
}

impl ReasonStatus {
    pub(super) fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Valid, _) | (_, Self::Valid) => Self::Valid,
            (Self::MissingPrevents, _) | (_, Self::MissingPrevents) => Self::MissingPrevents,
            (Self::MissingWriter, _) | (_, Self::MissingWriter) => Self::MissingWriter,
            (Self::MissingBoth, _) | (_, Self::MissingBoth) => Self::MissingBoth,
            (Self::Absent, Self::Absent) => Self::Absent,
        }
    }
}

pub(super) fn parse_reason_text(body: &str) -> ReasonStatus {
    let cleaned = strip_block_stars(body);
    let lower = cleaned.to_ascii_lowercase();
    let mut best = ReasonStatus::Absent;
    let mut offset = 0;
    while let Some(rel) = find_label(&lower[offset..], TAG) {
        let start = offset + rel + TAG.len();
        best = best.merge(parse_after_tag(&cleaned[start..], &lower[start..]));
        if best == ReasonStatus::Valid {
            return best;
        }
        offset = start;
    }
    best
}

fn parse_after_tag(text: &str, lower: &str) -> ReasonStatus {
    let prevents = find_label(lower, PREVENTS);
    let writer = find_label(lower, WRITER);
    let has_prevents = prevents.is_some_and(|start| {
        let val_start = start + PREVENTS.len();
        let val_end = next_boundary(text.len(), val_start, writer);
        has_content(&text[val_start..val_end])
    });
    let has_writer = writer.is_some_and(|start| {
        let val_start = start + WRITER.len();
        let val_end = next_boundary(text.len(), val_start, prevents);
        has_content(&text[val_start..val_end])
    });
    match (has_prevents, has_writer) {
        (true, true) => ReasonStatus::Valid,
        (false, true) => ReasonStatus::MissingPrevents,
        (true, false) => ReasonStatus::MissingWriter,
        (false, false) => ReasonStatus::MissingBoth,
    }
}

fn next_boundary(len: usize, after: usize, other: Option<usize>) -> usize {
    other.filter(|&position| position >= after).unwrap_or(len)
}

fn has_content(slice: &str) -> bool {
    let trimmed = slice
        .trim_matches(|ch: char| ch.is_whitespace() || matches!(ch, ';' | ',' | '-' | ':' | '|'));
    trimmed.chars().any(char::is_alphanumeric)
}

fn find_label(lower: &str, label: &str) -> Option<usize> {
    let bytes = lower.as_bytes();
    let mut offset = 0;
    while let Some(rel) = lower[offset..].find(label) {
        let pos = offset + rel;
        let boundary = pos == 0
            || !matches!(
                bytes[pos - 1],
                b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-'
            );
        if boundary {
            return Some(pos);
        }
        offset = pos + 1;
    }
    None
}

fn strip_block_stars(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    for (index, line) in body.lines().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix('*') {
            out.push_str(rest.trim_start_matches('*'));
        } else {
            out.push_str(trimmed);
        }
    }
    out
}

pub(super) fn reason_message(
    file: &str,
    line: usize,
    category: &str,
    status: ReasonStatus,
) -> String {
    match status {
        ReasonStatus::MissingPrevents => format!(
            "{file}:{line}: PostgreSQL {category} lock has a `lock-reason:` comment missing `prevents:`. Name the outcome the lock prevents alongside `writer:`, for example: `lock-reason: prevents: <outcome>; writer: <competing writer>`"
        ),
        ReasonStatus::MissingWriter => format!(
            "{file}:{line}: PostgreSQL {category} lock has a `lock-reason:` comment missing `writer:`. Name the competing writer alongside `prevents:`, for example: `lock-reason: prevents: <outcome>; writer: <competing writer>`"
        ),
        ReasonStatus::MissingBoth => format!(
            "{file}:{line}: PostgreSQL {category} lock has a `lock-reason:` comment missing both `prevents:` and `writer:`. Name the outcome the lock prevents and the competing writer, for example: `lock-reason: prevents: <outcome>; writer: <competing writer>`"
        ),
        ReasonStatus::Absent | ReasonStatus::Valid => format!(
            "{file}:{line}: PostgreSQL {category} lock has no `lock-reason:` comment. Add one in the same statement or on the line directly above naming what the lock prevents and the competing writer, for example: `lock-reason: prevents: <outcome>; writer: <competing writer>`"
        ),
    }
}
