use super::super::{Expr, Stmt};
use super::val::{Path, Val};
use super::{Machine, MAX_EMBEDDED_SQL_VARIANTS};

impl Machine<'_> {
    /// `for (const name of iterable) body`, unrolled once per element. Only
    /// an array of known elements (at most [`MAX_EMBEDDED_SQL_VARIANTS`]) is
    /// followed; anything else could iterate an unbounded number of times.
    pub(super) fn for_of(
        &mut self,
        name: &str,
        iterable: &Expr,
        body: &[Stmt],
        path: Path,
    ) -> Option<Vec<Path>> {
        let mut out = Vec::new();
        for (path, value) in self.eval(iterable, path)? {
            let Val::Arr(elements) = value else {
                return None;
            };
            if elements.len() > MAX_EMBEDDED_SQL_VARIANTS {
                return None;
            }
            let mut paths = vec![path];
            for element in elements {
                let mut next = Vec::new();
                for mut path in paths {
                    if path.returned.is_some() {
                        next.push(path);
                        continue;
                    }
                    path.push_scope();
                    path.bind(name, element.clone());
                    for mut done in self.block(body, vec![path])? {
                        done.pop_scope();
                        next.push(done);
                    }
                }
                if next.len() > MAX_EMBEDDED_SQL_VARIANTS {
                    return None;
                }
                paths = next;
            }
            out.extend(paths);
        }
        (out.len() <= MAX_EMBEDDED_SQL_VARIANTS).then_some(out)
    }
}
