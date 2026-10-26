//! Distinguish known SQL data operands from opaque runtime executor escapes.
use super::*;
use oxc_ast::ast::SimpleAssignmentTarget;
#[derive(Clone, Copy, Default)]
pub(super) struct Context {
    call: Option<Span>,
    argument: Option<Span>,
    tag: Option<usize>,
    bind: Option<usize>,
    invocation: Option<Span>,
    callee: Option<Span>,
    escape_argument: Option<Span>,
}
enum Owner {
    Call(Span),
    Bind(usize),
    AssignmentReceiver,
}
#[derive(Default)]
pub(super) struct Metadata {
    context: Context,
    references: FxHashMap<Span, Owner>,
    wanted: FxHashSet<Span>,
    escapes: FxHashMap<Span, Span>,
    escape_arguments: FxHashMap<Span, Span>,
    callee_uses: FxHashMap<Span, Span>,
}
impl Metadata {
    pub fn new(
        module: &crate::codebase::ts_source::module_facts::TypeScriptModuleFacts,
        selections: &[Selection],
    ) -> Self {
        let ids = selections
            .iter()
            .flat_map(|selection| {
                selection
                    .selected
                    .iter()
                    .chain(&selection.typed)
                    .chain(&selection.unsupported)
                    .chain(&selection.namespaces)
            })
            .copied()
            .collect::<FxHashSet<_>>();
        let wanted = module
            .bindings
            .iter()
            .filter(|binding| ids.contains(&binding.id))
            .flat_map(|binding| {
                binding
                    .references
                    .iter()
                    .filter(|usage| usage.runtime)
                    .map(|usage| (usage.span.start, usage.span.end))
            })
            .chain(
                selections
                    .iter()
                    .flat_map(|selection| selection.runtime.iter().copied()),
            )
            .collect();
        Self {
            wanted,
            ..Default::default()
        }
    }
    pub fn assignment_receiver(&mut self, target: &SimpleAssignmentTarget<'_>) {
        // Only the receiver is a write; computed keys and RHS expressions keep their uses.
        let receiver = match target {
            SimpleAssignmentTarget::StaticMemberExpression(member) => Some(&member.object),
            SimpleAssignmentTarget::ComputedMemberExpression(member) => Some(&member.object),
            _ => None,
        };
        if let Some(Expression::Identifier(receiver)) = receiver.map(unwrap_ts_wrappers) {
            self.references
                .insert(span(receiver.span), Owner::AssignmentReceiver);
        }
    }
    pub fn track_alias(&mut self, at: Span) {
        self.wanted.insert(at);
    }
    pub fn line(&self, at: &Span, source: &str) -> u32 {
        let start = self.escapes.get(at).unwrap_or(at).0;
        crate::codebase::ts_source::byte_offset_to_line(source, start as usize)
    }
    pub fn aggregate(&mut self) -> Context {
        let previous = self.context;
        self.context.argument = None;
        previous
    }
    pub fn enter_call(
        &mut self,
        call: Option<Span>,
        invocation: Option<Span>,
        callee: Option<Span>,
    ) -> Context {
        std::mem::replace(
            &mut self.context,
            Context {
                call,
                invocation,
                callee,
                ..Default::default()
            },
        )
    }
    pub fn enter_tag(&mut self, tag: Option<usize>, invocation: Span) -> Context {
        std::mem::replace(
            &mut self.context,
            Context {
                tag,
                invocation: tag.is_none().then_some(invocation),
                ..Default::default()
            },
        )
    }
    pub fn bind_expression(&mut self) -> Context {
        let previous = self.context;
        if let Some(tag) = self.context.tag {
            self.context.bind = Some(tag);
        }
        previous
    }
    pub fn argument(&mut self, index: usize) -> Context {
        let previous = self.context;
        self.context.argument = (index > 0).then_some(self.context.call).flatten();
        self.context.escape_argument = self.context.invocation;
        previous
    }
    pub fn restore(&mut self, previous: Context) {
        self.context = previous;
    }
    pub fn reference(&mut self, at: Span) {
        if !self.wanted.contains(&at) {
            return;
        }
        if let Some(invocation) = self.context.invocation {
            self.escapes.insert(at, invocation);
            if self
                .context
                .callee
                .is_some_and(|callee| callee.0 <= at.0 && at.1 <= callee.1)
            {
                self.callee_uses.insert(at, invocation);
            }
        }
        let owner = self
            .context
            .argument
            .map(Owner::Call)
            .or_else(|| self.context.bind.map(Owner::Bind));
        if let Some(owner) = owner {
            self.references.insert(at, owner);
        }
        if let Some(owner) = self.context.escape_argument {
            self.escape_arguments.insert(at, owner);
        }
    }
    pub fn belongs_to(
        &self,
        at: &Span,
        direct: &FxHashSet<Span>,
        tags: &template_binds::TemplateBinds,
    ) -> bool {
        match self.references.get(at) {
            Some(Owner::Call(call)) => direct.contains(call),
            // A later tag write invalidates bind classification request-wide.
            Some(Owner::Bind(id)) => tags.is_valid(*id),
            Some(Owner::AssignmentReceiver) => true,
            None => false,
        }
    }
    pub fn runtime_callee(&self, at: &Span) -> Option<Span> {
        self.callee_uses.get(at).copied()
    }
    pub fn runtime_callsite(&self, at: &Span) -> Option<Span> {
        self.callee_uses
            .get(at)
            .copied()
            .or_else(|| self.escape_arguments.get(at).copied())
    }
}
impl Syntax<'_> {
    // Opaque invocations and deferred bodies own their uses independently of
    // the surrounding executor's data; those escapes remain unknown.
    pub(super) fn opaque_metadata(&mut self, visit: impl FnOnce(&mut Self)) {
        let previous = self.metadata.enter_call(None, None, None);
        visit(self);
        self.metadata.restore(previous);
    }
}
