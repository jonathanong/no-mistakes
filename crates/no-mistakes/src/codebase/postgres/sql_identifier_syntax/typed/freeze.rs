use super::*;
use oxc_span::GetSpan;
mod collect;
mod solve;
mod value;
use value::{Field, Node, Taint};
pub(super) struct Sources<'a> {
    pub initial: &'a FxHashSet<usize>,
    pub proven: &'a FxHashSet<usize>,
    pub factories: &'a FxHashSet<usize>,
    pub writes: &'a FxHashSet<usize>,
    pub candidates: &'a FxHashSet<Span>,
    pub data: &'a FxHashSet<Span>,
    pub producers: &'a FxHashSet<usize>,
    pub handled: &'a FxHashSet<Span>,
}
#[derive(Clone, Copy, Default)]
pub(super) struct Frame {
    owner: Option<usize>,
    scope: Option<Span>,
    this_scope: Option<Span>,
}
#[derive(Default)]
pub(super) struct Proof {
    pub bindings: FxHashSet<usize>,
    pub runtime: FxHashSet<Span>,
    pub transports: FxHashSet<Span>,
}
#[derive(Default)]
pub(super) struct Tracker {
    clean: bool,
    enabled: bool,
    global_escape: bool,
    globals: FxHashSet<Span>,
    candidates: FxHashSet<Span>,
    write_roots: FxHashSet<Span>,
    nodes: FxHashMap<Span, Node>,
    order: Vec<Span>,
    flows: Vec<(usize, Span, Vec<Option<String>>)>,
    calls: Vec<(Span, Vec<Span>)>,
    returns: Vec<(usize, Span)>,
    aliases: Vec<(usize, usize)>,
    owners: FxHashMap<Span, usize>,
    frame: Frame,
    methods: FxHashMap<(Span, String), Span>,
    this_reads: FxHashMap<Span, Vec<Span>>,
    member_reads: FxHashMap<Span, Vec<Span>>,
    returned_scopes: FxHashSet<Span>,
    barriers: Vec<Span>,
    stores: Vec<(usize, Span)>,
    tests: Vec<Span>,
}
impl Tracker {
    pub fn new(clean: bool, enabled: bool) -> Self {
        Self {
            clean,
            enabled,
            ..Self::default()
        }
    }
    pub fn enter(&mut self, at: Span, id: Option<usize>, lexical: bool) -> Frame {
        let frame = Frame {
            owner: self.owners.get(&at).copied().or(id),
            scope: Some(at),
            this_scope: if lexical {
                self.frame.this_scope
            } else {
                Some(at)
            },
        };
        std::mem::replace(&mut self.frame, frame)
    }
    pub fn restore(&mut self, frame: Frame) {
        self.frame = frame;
    }
    pub fn test(&mut self, expression: &Expression<'_>) {
        if !self.enabled {
            return;
        }
        self.tests.push(span(expression.span()));
    }
    pub fn returned(&mut self, expression: &Expression<'_>) {
        if !self.enabled {
            return;
        }
        if let Some(scope) = self.frame.scope {
            self.returned_scopes.insert(scope);
        }
        if let Some(id) = self.frame.owner {
            self.returns.push((id, span(expression.span())));
        }
    }
}
