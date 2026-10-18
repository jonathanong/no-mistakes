//! Values and per-path state of one helper execution.

use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;

/// A string and, when the helper itself wrote it, its physical provenance.
#[derive(Clone, PartialEq, Eq)]
pub(super) struct Text {
    pub(super) value: String,
    pub(super) prov: Option<RecoveredSql>,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct Obj {
    pub(super) props: Vec<(String, Val)>,
    /// Other properties exist whose values are not known.
    pub(super) open: bool,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) enum Val {
    Str(Text),
    Bool(bool),
    Null,
    Undef,
    /// A scalar the call site passed whose value is not known (never SQL).
    Unknown,
    /// A name the helper does not bind: it could be anything, so it can be
    /// tested but never bound, spliced or appended.
    Free,
    Obj(Obj),
    /// An array whose elements are all known.
    Arr(Vec<Val>),
    /// A builder, by heap index.
    Sql(usize),
}

#[derive(Clone)]
pub(super) struct Heap {
    pub(super) text: RecoveredSql,
    /// The parameter that received this builder from the caller, if any.
    /// Its prior content is invisible here, so it can be appended to but
    /// never read back as a fragment.
    pub(super) param: Option<usize>,
}

/// One execution path: a finite alternative with its own mutable state.
#[derive(Clone)]
pub(super) struct Path {
    scopes: Vec<Vec<(String, Val)>>,
    pub(super) heap: Vec<Heap>,
    /// Outcomes of unknown tests on parameters, so a repeated test agrees.
    decided: Vec<(String, bool)>,
    /// `Some` once the path returned.
    pub(super) returned: Option<Val>,
}

pub(super) fn empty_sql() -> RecoveredSql {
    RecoveredSql {
        sql: String::new(),
        line: 1,
        positions: Vec::new(),
        origins: Vec::new(),
    }
}

impl Path {
    pub(super) fn new() -> Self {
        Self {
            scopes: vec![Vec::new()],
            heap: Vec::new(),
            decided: Vec::new(),
            returned: None,
        }
    }

    pub(super) fn push_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    pub(super) fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub(super) fn bind(&mut self, name: &str, value: Val) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.push((name.to_string(), value));
        }
    }

    pub(super) fn lookup(&self, name: &str) -> Option<&Val> {
        self.scopes
            .iter()
            .rev()
            .flat_map(|scope| scope.iter().rev())
            .find(|(bound, _)| bound == name)
            .map(|(_, value)| value)
    }

    pub(super) fn new_builder(&mut self, text: RecoveredSql, param: Option<usize>) -> Val {
        self.heap.push(Heap { text, param });
        Val::Sql(self.heap.len() - 1)
    }

    pub(super) fn decision(&self, key: &str) -> Option<bool> {
        self.decided
            .iter()
            .find(|(known, _)| known == key)
            .map(|(_, value)| *value)
    }

    pub(super) fn decide(&mut self, key: Option<String>, value: bool) {
        if let Some(key) = key {
            self.decided.push((key, value));
        }
    }
}
