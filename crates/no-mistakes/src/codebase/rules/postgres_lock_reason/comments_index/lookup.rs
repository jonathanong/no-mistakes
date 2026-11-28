use super::super::comment::{parse_reason_text, ReasonStatus};
use super::lexers::is_utf8_cont;
use super::{CommentIndex, CommentSpan};

impl CommentIndex {
    pub(in super::super) fn status_for_ts_line(&self, line: usize) -> ReasonStatus {
        let mut status = self.eval_contiguous_above(line);
        let same_line: Vec<_> = self
            .comments
            .iter()
            .filter(|c| c.start_line <= line && c.end_line >= line)
            .collect();
        status = status.merge(eval_group(&same_line));
        status
    }

    pub(in super::super) fn status_for_ts_offset(&self, line: usize, offset: u32) -> ReasonStatus {
        let Some(col) = self.col_for_offset(line, offset as usize) else {
            return self.status_for_ts_line(line);
        };
        let end_line = self.call_end_line(line, offset as usize);
        self.status_for_ts_span(line, col, end_line)
    }

    pub(in super::super) fn status_for_ts_position(&self, line: usize, col: usize) -> ReasonStatus {
        self.status_for_ts_span(line, col.max(1), line)
    }

    fn status_for_ts_span(&self, line: usize, col: usize, end_line: usize) -> ReasonStatus {
        let split = self.semicolons.partition_point(|&pos| pos < (line, col));
        let prev_semi = split
            .checked_sub(1)
            .map(|idx| self.semicolons[idx])
            .filter(|&(l, _)| l == line);
        let next_semi = self
            .semicolons
            .get(split)
            .copied()
            .filter(|&(l, _)| l <= end_line);
        let after_prev = prev_semi.map_or((1, 1), |(l, c)| (l, c + 1));
        let trailing_limit = match next_semi {
            Some((l, c)) if self.last_code_col(l) > c => (l, c),
            Some((l, _)) => (l, usize::MAX),
            None => (end_line, usize::MAX),
        };
        let in_stmt: Vec<_> = self
            .comments
            .iter()
            .filter(|c| {
                c.start_line <= end_line
                    && c.end_line >= line
                    && (c.end_line, c.end_col) >= after_prev
                    && (c.start_line, c.start_col) <= trailing_limit
            })
            .collect();
        let mut status = eval_group(&in_stmt);
        if prev_semi.is_none() {
            status = status.merge(self.eval_contiguous_above(line));
        }
        status
    }

    fn col_for_offset(&self, line: usize, offset: usize) -> Option<usize> {
        let start = *self.line_starts.get(line.checked_sub(1)?)?;
        let end = self
            .line_starts
            .get(line)
            .copied()
            .unwrap_or(self.source_bytes.len());
        if offset < start || offset > end {
            return None;
        }
        Some(
            self.source_bytes[start..offset]
                .iter()
                .filter(|&&b| !is_utf8_cont(b))
                .count()
                + 1,
        )
    }

    pub(in super::super) fn status_for_sql_statement(
        &self,
        start_line: usize,
        start_col: usize,
        end_line: usize,
    ) -> ReasonStatus {
        let split = self
            .semicolons
            .partition_point(|&pos| pos < (start_line, start_col));
        let prev_semi = split.checked_sub(1).map(|idx| self.semicolons[idx]);
        let next_semi = self.semicolons.get(split).copied();
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

    pub(in super::super) fn status_for_sql_line(&self, line: usize, col: usize) -> ReasonStatus {
        let split = self.semicolons.partition_point(|&pos| pos < (line, col));
        let prev = split.checked_sub(1).map(|idx| self.semicolons[idx]);
        let next = self.semicolons.get(split).copied();
        let start_line = (1..=line)
            .rev()
            .take_while(|&l| {
                prev.is_none_or(|(pl, pc)| l > pl || (l == pl && self.last_code_col(pl) > pc))
            })
            .filter(|&l| self.is_code_line(l))
            .last()
            .unwrap_or(line);
        let start_col = prev
            .filter(|&(l, _)| l == start_line)
            .map_or_else(|| self.first_code_col(start_line).max(1), |(_, c)| c + 1);
        let end_line = next.map_or(self.first_code_col.len().saturating_sub(1), |(l, _)| l);
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
}

fn eval_group(comments: &[&CommentSpan]) -> ReasonStatus {
    let mut best = ReasonStatus::Absent;
    let mut index = 0;
    while index < comments.len() {
        let comment = comments[index];
        if comment.is_line {
            let mut joined = comment.body.clone();
            let mut prev_line = comment.end_line;
            index += 1;
            while index < comments.len()
                && comments[index].is_line
                && comments[index].start_line == prev_line + 1
            {
                joined.push('\n');
                joined.push_str(&comments[index].body);
                prev_line = comments[index].end_line;
                index += 1;
            }
            best = best.merge(parse_reason_text(&joined));
        } else {
            best = best.merge(parse_reason_text(&comment.body));
            index += 1;
        }
        if best == ReasonStatus::Valid {
            return best;
        }
    }
    best
}
