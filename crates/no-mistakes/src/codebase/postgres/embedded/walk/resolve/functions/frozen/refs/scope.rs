use crate::fx::FxHashMap;
use oxc_syntax::scope::ScopeFlags;

#[derive(Default)]
pub(super) struct Scopes {
    nodes: Vec<Scope>,
    current: usize,
    binding_scopes: FxHashMap<u32, usize>,
}

struct Scope {
    parent: usize,
    var_scope: bool,
    bindings: u32,
}

impl Scopes {
    pub(super) fn enter(&mut self, flags: ScopeFlags) {
        let id = self.nodes.len();
        self.nodes.push(Scope {
            parent: self.current,
            var_scope: flags.is_var(),
            bindings: 0,
        });
        self.current = id;
    }

    pub(super) fn leave(&mut self) {
        self.current = self.nodes[self.current].parent;
    }

    pub(super) fn current(&self) -> usize {
        self.current
    }

    pub(super) fn var_scope(&self) -> usize {
        let mut scope = self.current;
        while !self.nodes[scope].var_scope {
            scope = self.nodes[scope].parent;
        }
        scope
    }

    pub(super) fn assign_binding_scope(&mut self, start: u32, scope: usize) {
        self.binding_scopes.insert(start, scope);
    }

    pub(super) fn bind(&mut self, start: u32) {
        let scope = self.binding_scopes.remove(&start).unwrap_or(self.current);
        self.nodes[scope].bindings += 1;
    }

    pub(super) fn bind_declaration(&mut self, scope: usize) {
        self.nodes[scope].bindings += 1;
    }

    pub(super) fn root_is_unique(&self) -> bool {
        self.nodes[0].bindings == 1
    }

    pub(super) fn resolves_to_root(&self, mut scope: usize) -> bool {
        while scope != 0 {
            if self.nodes[scope].bindings != 0 {
                return false;
            }
            scope = self.nodes[scope].parent;
        }
        true
    }
}
