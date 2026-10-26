use super::*;
use std::collections::BTreeSet;
mod paths;
const PATH_LIMIT: usize = 16;
const VARIANT_LIMIT: usize = 64;
#[derive(Clone, Default, Eq, PartialEq)]
pub(super) struct Taint {
    pub paths: BTreeSet<Vec<String>>,
    pub open: BTreeSet<Vec<String>>,
    pub producers: BTreeSet<(Vec<String>, usize)>,
    pub objects: BTreeSet<(Vec<String>, Span)>,
    pub broad: bool,
    pub widened_factory: bool,
    pub unproven_executor: bool,
    pub derived: bool,
}
impl Taint {
    pub fn executor() -> Self {
        Self {
            paths: [Vec::new(), vec!["query".into()]].into(),
            open: [Vec::new()].into(),
            derived: true,
            unproven_executor: true,
            ..Self::default()
        }
    }
    pub fn contract_executor() -> Self {
        Self {
            unproven_executor: false,
            ..Self::executor()
        }
    }
    pub fn latent(&self) -> bool {
        self.open.contains(&Vec::new())
    }
    pub fn present(&self) -> bool {
        self.broad || !self.paths.is_empty() || !self.producers.is_empty()
    }
    pub fn executable(&self) -> bool {
        self.broad || self.paths.contains(&Vec::new())
    }
    pub fn factory_value(&self) -> bool {
        !self.broad
            && !self.widened_factory
            && !self.executable()
            && self.producers.iter().any(|(path, _)| path.is_empty())
    }
    pub fn executor_leaf(&self) -> bool {
        !self.unproven_executor
            && !self.broad
            && !self.widened_factory
            && self.producers.is_empty()
            && self.objects.is_empty()
            && self.paths.contains(&Vec::new())
            && self
                .paths
                .iter()
                .all(|path| path.is_empty() || path == &["query"])
            && self.open.iter().all(Vec::is_empty)
    }
    pub fn widen(&mut self) {
        self.widened_factory |= !self.producers.is_empty();
        self.broad = true;
        self.paths.clear();
        self.open.clear();
        self.producers.clear();
    }
    pub fn producer(id: usize) -> Self {
        Self {
            producers: [(Vec::new(), id)].into(),
            derived: true,
            ..Self::default()
        }
    }
    pub fn join(&mut self, other: Self) -> bool {
        let before = self.clone();
        self.derived |= other.derived;
        self.unproven_executor |= other.unproven_executor;
        self.widened_factory |= other.widened_factory;
        if self.broad || other.broad {
            self.widened_factory |= !self.producers.is_empty() || !other.producers.is_empty();
        }
        self.broad |= other.broad;
        if !self.broad {
            self.paths.extend(other.paths);
            self.open.extend(other.open);
            self.producers.extend(other.producers);
            self.objects.extend(other.objects);
        }
        if self.paths.len() + self.open.len() + self.producers.len() > VARIANT_LIMIT {
            self.broad = true;
        }
        if self.broad {
            self.widen();
        }
        if self.objects.len() > VARIANT_LIMIT {
            self.objects.clear();
            if self.present() {
                self.widen();
            }
        }
        *self != before
    }
}
#[derive(Clone)]
pub(super) enum Field {
    Key(String),
    Spread,
    Unknown,
}
#[derive(Clone)]
pub(super) enum Node {
    Scalar,
    This(Option<Span>),
    Data(Vec<Span>),
    Ref(usize, Span),
    Copy(Span),
    Invoke(usize),
    InvokeValue(Span),
    Object(Vec<(Field, Span)>),
    Read(Span, Option<String>),
    Freeze(Span),
    Union(Vec<Span>),
}
impl Node {
    pub fn children(&self) -> Vec<Span> {
        match self {
            Self::Union(values) | Self::Data(values) => values.clone(),
            Self::Object(fields) => fields.iter().map(|(_, at)| *at).collect(),
            Self::Copy(at) | Self::Read(at, _) | Self::Freeze(at) => vec![*at],
            _ => Vec::new(),
        }
    }
}
