use super::comment::ReasonStatus;

mod lexers;
mod scan;
use scan::eval_group;

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
}

impl CommentIndex {
    pub(super) fn for_sql(source: &str) -> Self {
        scan::for_sql(source)
    }

    pub(super) fn for_ts(source: &str) -> Self {
        scan::for_ts(source)
    }

    pub(super) fn status_for_ts_line(&self, line: usize) -> ReasonStatus {
        let mut status = self.eval_contiguous_above(line);
        let same_line: Vec<_> = self
            .comments
            .iter()
            .filter(|c| c.start_line <= line && c.end_line >= line)
            .collect();
        status = status.merge(eval_group(&same_line));
        status
    }

    pub(super) fn status_for_sql_statement(
        &self,
        start_line: usize,
        start_col: usize,
        end_line: usize,
    ) -> ReasonStatus {
        let prev_semi = self
            .semicolons
            .partition_point(|&pos| pos < (start_line, start_col))
            .checked_sub(1)
            .map(|idx| self.semicolons[idx]);
        let next_semi = self
            .semicolons
            .get(
                self.semicolons
                    .partition_point(|&pos| pos < (start_line, start_col)),
            )
            .copied();
        let after_prev = prev_semi.map_or((1, 1), |(l, c)| (l, c + 1));
        let trailing_limit = match next_semi {
            Some((l, c)) if l == end_line => {
                if self.last_code_col(l) > c {
                    (l, c)
                } else {
                    (l, usize::MAX)
                }
            }
            Some(pos) => pos,
            None => (end_line, usize::MAX),
        };
        let in_stmt: Vec<_> = self
            .comments
            .iter()
            .filter(|c| {
                (c.end_line, c.end_col) >= after_prev
                    && (c.start_line, c.start_col) <= trailing_limit
                    && c.end_line >= start_line
            })
            .collect();
        let mut status = eval_group(&in_stmt);
        let first_on_line = prev_semi.is_none_or(|(l, _)| l < start_line)
            && self.first_code_col(start_line) >= start_col;
        if first_on_line {
            status = status.merge(self.eval_contiguous_above(start_line));
        }
        status
    }

    pub(super) fn status_for_sql_line(&self, line: usize, col: usize) -> ReasonStatus {
        let prev = self
            .semicolons
            .partition_point(|&pos| pos < (line, col))
            .checked_sub(1)
            .map(|idx| self.semicolons[idx]);
        let next = self
            .semicolons
            .get(self.semicolons.partition_point(|&pos| pos < (line, col)))
            .copied();
        let start_line = (1..=line)
            .rev()
            .take_while(|&l| prev.is_none_or(|(pl, _)| l > pl))
            .filter(|&l| self.is_code_line(l))
            .last()
            .unwrap_or(line);
        let start_col = prev
            .filter(|&(l, _)| l == start_line)
            .map_or_else(|| self.first_code_col(start_line).max(1), |(_, c)| c + 1);
        let end_line = next.map_or(line, |(l, _)| l);
        self.status_for_sql_statement(start_line, start_col, end_line)
    }

    fn eval_contiguous_above(&self, target_line: usize) -> ReasonStatus {
        let mut cursor = target_line.saturating_sub(1);
        let mut collected = Vec::new();
        while cursor >= 1 && !self.is_code_line(cursor) {
            let mut matched = false;
            for comment in &self.comments {
                if comment.start_line <= cursor && comment.end_line >= cursor {
                    collected.push(comment);
                    matched = true;
                }
            }
            if !matched {
                break;
            }
            cursor -= 1;
        }
        collected.sort_by_key(|c| (c.start_line, c.start_col));
        collected.dedup_by_key(|c| (c.start_line, c.start_col));
        eval_group(&collected)
    }

    fn empty(source: &str) -> Self {
        let lines = source.bytes().filter(|&b| b == b'\n').count() + 2;
        Self {
            comments: Vec::new(),
            first_code_col: vec![0; lines],
            last_code_col: vec![0; lines],
            semicolons: Vec::new(),
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
