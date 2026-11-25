#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DocumentSpan {
    pub(super) start_line: usize,
    pub(super) start_byte: usize,
    pub(super) end_byte: usize,
}

pub(super) fn document_spans(source: &str) -> Vec<DocumentSpan> {
    let mut spans: Vec<DocumentSpan> = Vec::new();
    let mut in_doc = false;
    let mut has_directives = false;
    let mut offset = 0usize;
    for (idx, raw_line) in source.split_inclusive('\n').enumerate() {
        let line_no = idx + 1;
        let line_start = offset;
        offset += raw_line.len();
        let line = raw_line.trim_end_matches(['\r', '\n']);
        let check = if idx == 0 {
            line.strip_prefix('\u{feff}').unwrap_or(line)
        } else {
            line
        };
        if is_doc_marker(check, "---") {
            if in_doc || !has_directives {
                if in_doc {
                    if let Some(prev) = spans.last_mut() {
                        prev.end_byte = line_start;
                    }
                }
                spans.push(DocumentSpan {
                    start_line: line_no,
                    start_byte: line_start,
                    end_byte: source.len(),
                });
            }
            in_doc = true;
            has_directives = false;
        } else if is_doc_marker(check, "...") {
            if in_doc || has_directives {
                if let Some(prev) = spans.last_mut() {
                    prev.end_byte = line_start;
                }
            }
            in_doc = false;
            has_directives = false;
        } else if check.trim().is_empty() || check.trim_start().starts_with('#') {
            continue;
        } else if !in_doc && check.starts_with('%') {
            if !has_directives {
                spans.push(DocumentSpan {
                    start_line: line_no,
                    start_byte: line_start,
                    end_byte: source.len(),
                });
                has_directives = true;
            }
        } else if !in_doc {
            if !has_directives {
                spans.push(DocumentSpan {
                    start_line: line_no,
                    start_byte: line_start,
                    end_byte: source.len(),
                });
            }
            in_doc = true;
            has_directives = false;
        }
    }
    spans
}

fn is_doc_marker(line: &str, marker: &str) -> bool {
    line.strip_prefix(marker)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with([' ', '\t']))
}
