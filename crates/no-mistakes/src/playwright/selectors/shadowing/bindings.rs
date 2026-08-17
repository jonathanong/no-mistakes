//! String-scan equivalents of the former identifier-shadowing regexes.

const DECL_KEYWORDS: [&str; 3] = ["const", "let", "var"];

pub(super) fn has_declaration(source: &str, name: &str) -> bool {
    for_each_decl_binding_start(source, |start| {
        is_identifier_at(source, start, name).then_some(())
    })
    .is_some()
}

pub(super) fn has_destructuring_declaration(source: &str, name: &str) -> bool {
    for_each_decl_binding_start(source, |start| destructure_end(source, start, name, ';')).is_some()
}

pub(super) fn function_destructure_binding_ends<'a>(
    source: &'a str,
    name: &'a str,
) -> impl Iterator<Item = usize> + 'a {
    FunctionDestructureEnds {
        source,
        name,
        search: 0,
    }
}

struct FunctionDestructureEnds<'a> {
    source: &'a str,
    name: &'a str,
    search: usize,
}

impl Iterator for FunctionDestructureEnds<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(at) = find_keyword(self.source, "function", self.search) {
            self.search = at + "function".len();
            let after_fn = &self.source[self.search..];
            let Some(paren) = after_fn.find('(') else {
                continue;
            };
            let params_start = self.search + paren + 1;
            if let Some(end) = destructure_end(self.source, params_start, self.name, ')') {
                self.search = end;
                return Some(end);
            }
        }
        None
    }
}

fn for_each_decl_binding_start<T>(
    source: &str,
    mut found: impl FnMut(usize) -> Option<T>,
) -> Option<T> {
    for keyword in DECL_KEYWORDS {
        let mut search = 0;
        while let Some(at) = find_keyword(source, keyword, search) {
            search = at + keyword.len();
            let Some(start) = skip_required_whitespace(source, search) else {
                continue;
            };
            if let Some(value) = found(start) {
                return Some(value);
            }
        }
    }
    None
}

fn destructure_end(source: &str, start: usize, name: &str, stop: char) -> Option<usize> {
    let rest = source.get(start..)?;
    let (open, close) = match rest.chars().next()? {
        '{' => ('{', '}'),
        '[' => ('[', ']'),
        _ => return None,
    };
    name_then_close(source, start + open.len_utf8(), name, close, stop)
}

fn name_then_close(
    source: &str,
    inner_start: usize,
    name: &str,
    close: char,
    stop: char,
) -> Option<usize> {
    let region_end = source[inner_start..]
        .find(stop)
        .map(|index| inner_start + index)
        .unwrap_or(source.len());
    let mut search = inner_start;
    while let Some(at) = find_identifier(source, name, search, region_end) {
        let after = at + name.len();
        if after < region_end {
            if let Some(close_rel) = source[after..region_end].find(close) {
                return Some(after + close_rel + close.len_utf8());
            }
        }
        search = at + 1;
    }
    None
}

fn find_keyword(source: &str, keyword: &str, start: usize) -> Option<usize> {
    find_identifier(source, keyword, start, source.len())
}

fn find_identifier(source: &str, name: &str, start: usize, end: usize) -> Option<usize> {
    if name.is_empty() || start >= end {
        return None;
    }
    let haystack = source.get(start..end)?;
    let mut search = 0;
    while let Some(rel) = haystack[search..].find(name) {
        let at = start + search + rel;
        if is_identifier_at(source, at, name) {
            return Some(at);
        }
        search += rel + 1;
    }
    None
}

fn skip_required_whitespace(source: &str, start: usize) -> Option<usize> {
    let rest = source.get(start..)?;
    let trimmed = rest.trim_start();
    (trimmed.len() < rest.len()).then_some(start + (rest.len() - trimmed.len()))
}

pub(super) fn is_identifier_at(source: &str, start: usize, name: &str) -> bool {
    let end = start + name.len();
    source.get(start..end) == Some(name) && is_identifier_span(source, start, end)
}

fn is_identifier_span(source: &str, start: usize, end: usize) -> bool {
    !source[..start]
        .chars()
        .next_back()
        .is_some_and(super::is_identifier_continue)
        && !source
            .get(end..)
            .and_then(|rest| rest.chars().next())
            .is_some_and(super::is_identifier_continue)
}
