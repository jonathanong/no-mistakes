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

// CommonMark HTML blocks of type 6. Types 1 (`script`, `pre`, `style`, and
// `textarea`) have different opening rules and are handled separately below.
const BLOCK_HTML_TAGS: &str = concat!(
    "address article aside base basefont blockquote body caption center col colgroup dd details ",
    "dialog dir div dl dt fieldset figcaption figure footer form frame frameset h1 h2 h3 h4 h5 ",
    "h6 head header hr html iframe legend li link main menu menuitem meta nav noframes ol optgroup ",
    "option p param search section summary table tbody td tfoot th ",
    "thead title tr track ul",
);

const RAW_TEXT_HTML_TAGS: &str = "script pre style textarea";

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
    looks_like_clear_mdx_jsx(line, 0..line.len())
        || looks_like_commonmark_interrupting_html_block_opener(line)
}

/// Returns whether `line` begins with a CommonMark HTML block type that can
/// interrupt a paragraph. Type 7 HTML blocks intentionally do not qualify.
fn looks_like_commonmark_interrupting_html_block_opener(line: &str) -> bool {
    line.starts_with("<!--")
        || line.starts_with("<?")
        || line.starts_with("<![CDATA[")
        || line
            .strip_prefix("<!")
            .and_then(|rest| rest.as_bytes().first())
            .is_some_and(u8::is_ascii_uppercase)
        || looks_like_html_tag_opener(line, RAW_TEXT_HTML_TAGS, false, false)
        || looks_like_html_tag_opener(line, BLOCK_HTML_TAGS, true, true)
}

fn looks_like_html_tag_opener(
    line: &str,
    tags: &str,
    allow_closing: bool,
    allow_self_closing: bool,
) -> bool {
    let Some(after_open) = line.strip_prefix('<') else {
        return false;
    };
    let (after_open, closing) = match after_open.strip_prefix('/') {
        Some(after_open) => (after_open, true),
        None => (after_open, false),
    };
    if closing && !allow_closing {
        return false;
    }
    let name_end = after_open
        .find(|character: char| !character.is_ascii_alphanumeric())
        .unwrap_or(after_open.len());
    let name = &after_open[..name_end];
    let delimiter = after_open.as_bytes().get(name_end).copied();
    let valid_delimiter = delimiter.is_none_or(|byte| byte.is_ascii_whitespace() || byte == b'>')
        || (allow_self_closing
            && delimiter == Some(b'/')
            && after_open.as_bytes().get(name_end + 1) == Some(&b'>'));
    if !valid_delimiter {
        return false;
    }
    tags.split_ascii_whitespace()
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
