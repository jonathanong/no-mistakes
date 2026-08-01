use std::ops::Range;

const STANDARD_HTML_TAGS: &str = concat!(
    "a abbr acronym address applet area article aside audio b base basefont bdi bdo big ",
    "blockquote body br button canvas caption center cite code col colgroup data datalist dd ",
    "del details dfn dialog dir div dl dt em embed fieldset figcaption figure font footer form ",
    "frame frameset h1 h2 h3 h4 h5 h6 head header hgroup hr html i iframe img input ins kbd ",
    "label legend li link main map mark marquee menu menuitem meta meter nav nobr noembed ",
    "noframes noscript object ol optgroup option output p param picture plaintext pre progress ",
    "q rb rp rt rtc ruby s samp script search section select slot small source span strike ",
    "strong style sub summary sup table tbody td template textarea tfoot th thead time title tr ",
    "track tt u ul var video wbr xmp",
);

const BLOCK_HTML_TAGS: &str = concat!(
    "address article aside base basefont blockquote body caption center col colgroup dd details ",
    "dialog dir div dl dt fieldset figcaption figure footer form frame frameset h1 h2 h3 h4 h5 ",
    "h6 head header hr html iframe legend li link main menu menuitem nav noframes ol optgroup ",
    "option p param pre script search section style summary table tbody td textarea tfoot th ",
    "thead title tr track ul",
);

pub(crate) fn looks_like_clear_mdx_jsx(source: &str, range: Range<usize>) -> bool {
    let block = source[range].trim_start();
    let Some(after_open) = block.strip_prefix('<') else {
        return false;
    };
    if after_open.starts_with('>') {
        return true;
    }

    let name_end = after_open
        .find(|character: char| {
            !character.is_ascii_alphanumeric() && !matches!(character, '_' | '-' | '.' | ':' | '$')
        })
        .unwrap_or(after_open.len());
    let name = &after_open[..name_end];
    let component_name = name
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_uppercase())
        && !STANDARD_HTML_TAGS
            .split_ascii_whitespace()
            .any(|tag| name.eq_ignore_ascii_case(tag));
    component_name || has_unquoted_jsx_expression_brace(after_open)
}

pub(super) fn looks_like_mdx_flow_boundary(line: &[u8]) -> bool {
    let indent = line.iter().take_while(|byte| **byte == b' ').count();
    if indent > 3 || line.get(indent) == Some(&b'\t') {
        return false;
    }
    let line = std::str::from_utf8(&line[indent..]).expect("MDX source line must remain UTF-8");
    looks_like_clear_mdx_jsx(line, 0..line.len()) || looks_like_block_html_tag(line)
}

fn looks_like_block_html_tag(line: &str) -> bool {
    let Some(after_open) = line.strip_prefix('<') else {
        return false;
    };
    let after_open = after_open.strip_prefix('/').unwrap_or(after_open);
    let name_end = after_open
        .find(|character: char| !character.is_ascii_alphanumeric())
        .unwrap_or(after_open.len());
    let name = &after_open[..name_end];
    let delimiter = after_open.as_bytes().get(name_end).copied();
    if delimiter.is_some_and(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b'/' | b'>')) {
        return false;
    }
    BLOCK_HTML_TAGS
        .split_ascii_whitespace()
        .any(|tag| name.eq_ignore_ascii_case(tag))
}

pub(super) fn has_unquoted_jsx_expression_brace(opening: &str) -> bool {
    let mut quote = None;
    for character in opening.chars() {
        if let Some(expected) = quote {
            if character == expected {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '{' => return true,
            '>' => return false,
            _ => {}
        }
    }
    false
}

#[cfg(test)]
#[path = "mdx_jsx/tests.rs"]
mod tests;
