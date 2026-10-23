//! Building SQL text: templates, appends, inlined fragments and binds.

use super::super::{Expr, Mark};
use super::val::{empty_sql, Path, Text, Val};
use super::Machine;
use crate::codebase::postgres::embedded::helpers::split::{compose, plain, Marker};
use crate::codebase::postgres::embedded::placeholders::internal_placeholder;
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;

/// A spliced literal keeps the physical line map exact only when it is one
/// line and cannot forge an internal marker.
fn splice_safe(text: &str) -> bool {
    !text.contains(['\n', '\r', '\u{10FFFE}', '\u{10FFFF}'])
}

impl Machine<'_> {
    pub(super) fn stamped(&self, text: &RecoveredSql) -> RecoveredSql {
        match self.stamp {
            Some(path) => text.clone().with_source_file(path),
            None => text.clone(),
        }
    }

    /// `value` as SQL text that reads from the expression at `mark`.
    pub(super) fn plain_text(&self, value: &str, mark: Mark) -> Option<RecoveredSql> {
        splice_safe(value).then(|| self.text_at(value.to_string(), mark))
    }

    fn text_at(&self, text: String, mark: Mark) -> RecoveredSql {
        plain(
            text,
            Marker {
                line: mark.line,
                path: self.stamp.map(|path| path.to_path_buf()),
                origin: mark.origin,
            },
        )
    }

    /// The placeholder a bound value leaves in the text.
    fn bind(&self, mark: Mark) -> RecoveredSql {
        self.text_at(internal_placeholder(1), mark)
    }

    /// The text a value contributes when appended or inlined as SQL.
    fn fragment(&self, value: &Val, path: &Path, mark: Mark) -> Option<RecoveredSql> {
        match value {
            Val::Str(Text {
                prov: Some(prov), ..
            }) => Some(prov.clone()),
            Val::Str(text) => self.plain_text(&text.value, mark),
            // A caller-owned builder's content is not visible to the helper.
            Val::Sql(id) if path.heap[*id].param.is_none() => Some(path.heap[*id].text.clone()),
            _ => None,
        }
    }

    /// Interleaves `pieces` with `parts`; `None` unless counts line up.
    fn weave(&self, pieces: &[RecoveredSql], parts: Vec<RecoveredSql>) -> Option<RecoveredSql> {
        let mut woven = Vec::with_capacity(pieces.len() * 2);
        let mut parts = parts.into_iter();
        for (index, piece) in pieces.iter().enumerate() {
            if index > 0 {
                woven.push(parts.next()?);
            }
            woven.push(self.stamped(piece));
        }
        Some(compose(woven).unwrap_or_else(empty_sql))
    }

    /// An untagged template: each interpolation must be a literal string.
    pub(super) fn eval_template(
        &mut self,
        pieces: &[RecoveredSql],
        holes: &[(Expr, Mark)],
        path: Path,
    ) -> Option<Vec<(Path, Val)>> {
        let exprs: Vec<&Expr> = holes.iter().map(|(expr, _)| expr).collect();
        let mut out = Vec::new();
        for (path, values) in self.eval_all(&exprs, path)? {
            let mut parts = Vec::with_capacity(values.len());
            for (value, (_, mark)) in values.iter().zip(holes) {
                let Val::Str(text) = value else { return None };
                parts.push(self.plain_text(&text.value, *mark)?);
            }
            let text = self.weave(pieces, parts)?;
            let value = Val::Str(Text {
                value: text.sql.clone(),
                prov: Some(text),
            });
            out.push((path, value));
        }
        Some(out)
    }

    /// A trusted-tag template: scalars bind, builders inline.
    pub(super) fn eval_sql(
        &mut self,
        pieces: &[RecoveredSql],
        holes: &[(Expr, Mark)],
        path: Path,
        parameterizes_all: bool,
    ) -> Option<Vec<(Path, Val)>> {
        let exprs: Vec<&Expr> = holes.iter().map(|(expr, _)| expr).collect();
        let mut out = Vec::new();
        for (mut path, values) in self.eval_all(&exprs, path)? {
            let mut parts = Vec::with_capacity(values.len());
            for (value, (_, mark)) in values.iter().zip(holes) {
                parts.push(match value {
                    Val::Sql(_) if !parameterizes_all => self.fragment(value, &path, *mark)?,
                    Val::Free if !parameterizes_all => return None,
                    _ => self.bind(*mark),
                });
            }
            let text = self.weave(pieces, parts)?;
            let value = path.new_builder(text, None);
            out.push((path, value));
        }
        Some(out)
    }

    /// `target.append(arg)`: raw text or an inlined fragment, in place.
    pub(super) fn eval_append(
        &mut self,
        target: &Expr,
        arg: &Expr,
        mark: Mark,
        path: Path,
    ) -> Option<Vec<(Path, Val)>> {
        let mut out = Vec::new();
        for (path, values) in self.eval_all(&[target, arg], path)? {
            let [Val::Sql(id), value] = values.as_slice() else {
                return None;
            };
            let part = self.fragment(value, &path, mark)?;
            let mut path = path;
            let base = std::mem::replace(&mut path.heap[*id].text, empty_sql());
            path.heap[*id].text = compose([base, part]).unwrap_or_else(empty_sql);
            out.push((path, Val::Sql(*id)));
        }
        Some(out)
    }
}
