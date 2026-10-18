#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommentKind {
    /// `<!-- ... -->`. Kind, not prefix length, so a future 4-byte prefix cannot collide.
    Html,
    Line {
        prefix_len: usize,
    },
}

impl CommentKind {
    fn line(prefix: &str) -> Self {
        Self::Line {
            prefix_len: prefix.len(),
        }
    }

    fn prefix_len(self) -> usize {
        match self {
            CommentKind::Html => "<!--".len(),
            CommentKind::Line { prefix_len } => prefix_len,
        }
    }
}

fn html_leading_comment_body(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("<!--")?;
    let end = rest.find("-->")?;
    rest[end + 3..]
        .trim()
        .is_empty()
        .then_some(rest[..end].trim())
}

fn html_directive_comment(line: &str, idx: usize) -> bool {
    let Some(rest) = line[idx..].strip_prefix("<!--") else {
        return false;
    };
    if !comment_prefix_can_start(line, idx) {
        return false;
    }
    let Some(end) = rest.find("-->") else {
        return false;
    };
    rest[..end].trim().starts_with("no-mistakes-disable-")
}

fn comment_body(line: &str, start: usize, kind: CommentKind) -> &str {
    let rest = line[start + kind.prefix_len()..].trim();
    match kind {
        CommentKind::Html => {
            let end = rest.find("-->").unwrap_or(rest.len());
            rest[..end].trim()
        }
        CommentKind::Line { .. } => rest,
    }
}
