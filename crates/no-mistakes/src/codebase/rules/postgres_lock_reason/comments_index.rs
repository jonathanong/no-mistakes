mod js_lexers;
mod lexers;
mod lookup;
mod scan;

#[derive(Debug)]
struct CommentSpan {
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
    body: String,
    is_line: bool,
}

pub(super) struct CommentIndex {
    comments: Vec<CommentSpan>,
    first_code_col: Vec<usize>,
    last_code_col: Vec<usize>,
    semicolons: Vec<(usize, usize)>,
    line_starts: Vec<usize>,
    source_bytes: Vec<u8>,
}

impl CommentIndex {
    pub(super) fn for_sql(source: &str) -> Self {
        scan::for_sql(source)
    }

    pub(super) fn for_ts(source: &str) -> Self {
        let mut index = scan::for_ts(source);
        index.source_bytes = source.as_bytes().to_vec();
        index.line_starts = std::iter::once(0)
            .chain(
                source
                    .bytes()
                    .enumerate()
                    .filter_map(|(i, b)| (b == b'\n').then_some(i + 1)),
            )
            .collect();
        index
    }

    fn empty(source: &str) -> Self {
        let lines = source.bytes().filter(|&b| b == b'\n').count() + 2;
        Self {
            comments: Vec::new(),
            first_code_col: vec![0; lines],
            last_code_col: vec![0; lines],
            semicolons: Vec::new(),
            line_starts: Vec::new(),
            source_bytes: Vec::new(),
        }
    }

    fn mark_code(&mut self, line: usize, col: usize) {
        if self.first_code_col[line] == 0 || col < self.first_code_col[line] {
            self.first_code_col[line] = col;
        }
        if col > self.last_code_col[line] {
            self.last_code_col[line] = col;
        }
    }

    fn is_code_line(&self, line: usize) -> bool {
        self.first_code_col.get(line).copied().unwrap_or(0) != 0
    }

    fn first_code_col(&self, line: usize) -> usize {
        self.first_code_col.get(line).copied().unwrap_or(0)
    }

    fn last_code_col(&self, line: usize) -> usize {
        self.last_code_col.get(line).copied().unwrap_or(0)
    }

    fn push_comment(
        &mut self,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
        body: &str,
        is_line: bool,
    ) {
        self.comments.push(CommentSpan {
            start_line,
            start_col,
            end_line,
            end_col,
            body: body.to_owned(),
            is_line,
        });
    }
}
