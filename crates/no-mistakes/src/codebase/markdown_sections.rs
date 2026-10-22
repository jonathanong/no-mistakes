use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag};

/// A parsed markdown section heading.
pub struct MarkdownSection {
    /// Heading level 1–6.
    pub level: u32,
    /// The heading text without any `#` prefix, e.g. `"Performance"`.
    pub heading: String,
    /// 1-based line number where the heading appears in the source.
    pub line: usize,
}

/// Parse all section headings from `content` and return them in document order.
pub fn parse_markdown_sections(content: &str) -> Vec<MarkdownSection> {
    let parser = Parser::new_ext(content, Options::all()).into_offset_iter();

    let mut sections = Vec::new();
    let mut current: Option<(u32, usize)> = None; // (level, byte_offset)
    let mut text_buf = String::new();

    for (event, range) in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                let level_u32 = heading_level_to_u32(level);
                let byte_offset = range.start;
                current = Some((level_u32, byte_offset));
                text_buf.clear();
            }
            Event::Text(text) | Event::Code(text) if current.is_some() => {
                text_buf.push_str(&text);
            }
            Event::End(pulldown_cmark::TagEnd::Heading(_)) => {
                if let Some((level, byte_offset)) = current.take() {
                    let line = byte_offset_to_line(content, byte_offset);
                    sections.push(MarkdownSection {
                        level,
                        heading: text_buf.clone(),
                        line,
                    });
                    text_buf.clear();
                }
            }
            _ => {}
        }
    }

    sections
}

/// Return `true` if any section heading in `content` matches `heading` exactly
/// (case-sensitive, no `#` prefix).
pub fn has_section(content: &str, heading: &str) -> bool {
    parse_markdown_sections(content)
        .iter()
        .any(|s| s.heading == heading)
}

/// Return `true` when `content` satisfies `required`.
///
/// An ATX `required` value such as `## Perf` must appear as a parsed heading
/// at that level. `# Perf` does not satisfy `## Perf`. A fenced or four-space
/// example of `## Perf` is not a heading. A bare heading name still matches
/// any level through [`has_section`]. Legacy leading-hash values that are not
/// valid ATX syntax strip their hashes and match at any level as well.
pub fn has_required_heading(content: &str, required: &str) -> bool {
    let required = required.trim();
    if let Some((level, heading)) = parse_atx_heading_spec(required) {
        return parse_markdown_sections(content)
            .iter()
            .any(|s| s.level == level && s.heading == heading);
    }
    has_section(content, required.trim_start_matches('#').trim())
}

/// Parse `## Perf` into `(2, "Perf")`. Requires CommonMark ATX shape: 1–6
/// hashes, then a space or tab before the heading text (or an empty heading).
fn parse_atx_heading_spec(required: &str) -> Option<(u32, &str)> {
    let hashes = required.bytes().take_while(|&b| b == b'#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = &required[hashes..];
    if rest.is_empty() {
        return Some((hashes as u32, ""));
    }
    if !rest.starts_with(' ') && !rest.starts_with('\t') {
        return None;
    }
    Some((hashes as u32, rest.trim()))
}

fn heading_level_to_u32(level: HeadingLevel) -> u32 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Convert a byte offset in `content` to a 1-based line number.
fn byte_offset_to_line(content: &str, byte_offset: usize) -> usize {
    let safe_offset = byte_offset.min(content.len());
    content[..safe_offset]
        .chars()
        .filter(|&c| c == '\n')
        .count()
        + 1
}

#[cfg(test)]
mod tests;
