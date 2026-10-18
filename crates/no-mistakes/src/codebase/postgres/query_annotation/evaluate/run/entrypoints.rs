use crate::codebase::postgres::query_annotation::{Expr, Function};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

// Lazy equivalent of collect: preserve base-before-key and source child order,
// global u32-start deduplication, and name-cycle exit timing across yields.
enum Frame<'a> {
    Value(&'a Expr),
    Values(std::slice::Iter<'a, Expr>),
    LeaveName(&'a str),
}
pub(super) struct Functions<'a> {
    globals: &'a BTreeMap<String, Expr>,
    visiting: BTreeSet<&'a str>,
    seen: BTreeSet<u32>,
    stack: Vec<Frame<'a>>,
}
impl<'a> Functions<'a> {
    pub(super) fn new(globals: &'a BTreeMap<String, Expr>) -> Self {
        Self {
            globals,
            visiting: BTreeSet::new(),
            seen: BTreeSet::new(),
            stack: Vec::new(),
        }
    }
    // Caller starts another global only after the previous traversal exhausted.
    pub(super) fn start(&mut self, value: &'a Expr) {
        self.stack.push(Frame::Value(value));
    }
}
impl Iterator for Functions<'_> {
    type Item = Arc<Function>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(frame) = self.stack.pop() {
            let value = match frame {
                Frame::Value(value) => value,
                Frame::Values(mut values) => {
                    if let Some(value) = values.next() {
                        self.stack.push(Frame::Values(values));
                        self.stack.push(Frame::Value(value));
                    }
                    continue;
                }
                Frame::LeaveName(name) => {
                    self.visiting.remove(name);
                    continue;
                }
            };
            match value {
                Expr::Function(function) => {
                    if self.seen.insert(function.start) {
                        return Some(Arc::clone(function));
                    }
                }
                Expr::Name(name) => {
                    if self.visiting.insert(name.as_str()) {
                        self.stack.push(Frame::LeaveName(name));
                        if let Some(value) = self.globals.get(name) {
                            self.stack.push(Frame::Value(value));
                        }
                    }
                }
                Expr::Record(values, _)
                | Expr::Object(values)
                | Expr::Array(values)
                | Expr::Container(values)
                | Expr::Sequence(values)
                | Expr::Alternatives(values) => {
                    self.stack.push(Frame::Values(values.iter()));
                }
                Expr::Spread(value)
                | Expr::Member(value, _)
                | Expr::Index(value, _)
                | Expr::OpaqueCallback(value)
                | Expr::Discard(value) => self.stack.push(Frame::Value(value)),
                Expr::ComputedMember(base, key) | Expr::Append(base, key) => {
                    self.stack.push(Frame::Value(key));
                    self.stack.push(Frame::Value(base));
                }
                _ => {}
            }
        }
        None
    }
}

#[cfg(test)]
#[path = "entrypoints_tests.rs"]
mod tests;
