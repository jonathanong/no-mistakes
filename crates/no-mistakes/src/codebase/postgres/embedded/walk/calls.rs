use super::variants::Recovered;
use super::{resolve, ScopeVisitor};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};
use oxc_ast_visit::walk;
use oxc_span::GetSpan;

impl<'a> ScopeVisitor<'a> {
    /// Remembers a call that is a whole expression statement, whose value is discarded.
    pub(super) fn note_statement_call(
        &mut self,
        statement: &oxc_ast::ast::ExpressionStatement<'a>,
    ) {
        if let oxc_ast::ast::Expression::CallExpression(call) =
            crate::codebase::ts_source::unwrap_ts_wrappers(&statement.expression)
        {
            self.statement_calls.insert(call.span.start);
        }
    }

    pub(super) fn visit_sql_call(&mut self, call: &CallExpression<'a>) {
        self.record_undeclared_executor_call(call);
        if self.head_calls_only {
            self.visit_head_call(call);
            return;
        }
        // `helper(builder)` on an imported builder helper appends its fixed
        // fragments to `builder`, so it composes like `builder.append(...)`.
        let statement_append = super::imported::builder_statement(self, call);
        // `q.append(a).append(b)`: the outer call applies every link in order.
        let consumed = self.consumed_links.contains(&call.span.start);
        let links = if statement_append.is_some() || consumed {
            None
        } else {
            chain(call)
        };
        if let Some(chain) = &links {
            self.consume_inner_links(chain);
        }
        let helper_effect = if statement_append.is_some() {
            (Vec::new(), None)
        } else {
            self.call_builder_effect(call)
        };
        let append_effect = match (&statement_append, &links) {
            (Some((name, _)), _) => self.hole_append_effect(name),
            (None, Some(chain)) => self.append_alias_effect(chain.links[0]),
            (None, None) if consumed => Vec::new(),
            (None, None) => self.append_alias_effect(call),
        };
        if self.suppress_nested_builder_fragments == 0 {
            if let Some(argument) = call.arguments.first() {
                if let Some(sql_text) = resolve::appended_builder_fragment(call, self) {
                    self.push_fragment(
                        crate::codebase::ts_source::byte_offset_to_line(
                            self.source,
                            argument.span().start as usize,
                        ),
                        Some(sql_text),
                        argument.as_expression(),
                        Some(call.span.start),
                    );
                } else if resolve::is_builder_append(call, self) {
                    self.push_fragment(
                        crate::codebase::ts_source::byte_offset_to_line(
                            self.source,
                            argument.span().start as usize,
                        ),
                        None,
                        argument.as_expression(),
                        Some(call.span.start),
                    );
                }
            }
        }
        let variants = match (&statement_append, &links) {
            (Some((name, hole)), _) => Some((
                name.clone(),
                self.appended_hole_variants(name, hole, call.span.start),
            )),
            (None, Some(chain)) => Some((chain.root.clone(), self.chain_variants(chain))),
            (None, None) if consumed => None,
            (None, None) => self
                .appended_variants(call)
                .map(|(name, values)| (name.to_string(), values)),
        };
        match (&statement_append, &links) {
            (Some((name, hole)), _) => resolve::apply_hole_append(self, name, hole),
            (None, Some(chain)) => self.apply_chain(chain),
            (None, None) if consumed => {}
            (None, None) => resolve::apply_append(self, call),
        }
        if let Some((name, values)) = variants {
            self.set_variants(&name, values);
        }
        let before = (self.calls.len(), self.pending_calls.len());
        resolve::record_executor_call(self, call);
        let arguments = self.begin_executor_arguments(call, before);
        self.record_argument_append_effects(&append_effect, call.span.start);
        let invocation = self.begin_argument_invocation(call);
        let exemptions = self.begin_safe_builder_arguments(
            call,
            &helper_effect.0,
            statement_append.as_ref().map(|(name, _)| name.as_str()),
        );
        walk::walk_call_expression(self, call);
        self.builder_reference_exemptions.truncate(exemptions);
        self.finish_argument_invocation(invocation);
        self.record_argument_helper_effects(&helper_effect.0, call.span.start);
        if let Some(arguments) = arguments {
            // Capture all argument evaluations before this invocation consumes
            // the builder. An unconfirmed relative callee can invalidate later
            // reuse without erasing its own pending executor arguments.
            self.finish_executor_arguments(call, arguments);
        }
        self.apply_call_builder_effect(helper_effect);
        self.apply_append_alias_effect(append_effect);
        // An unknown helper can replace a module namespace's `.query` export.
        self.revoke_namespace_arguments(&call.callee, &call.arguments);
    }
}

// A fluent statement `q.append(a).append(b)` mutates the tracked builder `q`
// once per link, in source order. The outermost call applies every link and
// marks the inner calls so the walk, which visits them next, does not apply
// them a second time.
pub(super) struct Chain<'a> {
    pub(super) root: String,
    /// Innermost link first.
    pub(super) links: Vec<&'a CallExpression<'a>>,
}

/// The links of `call` when it is `ident.append(..).append(..)…` with at
/// least two links. A single link is the ordinary direct append.
pub(super) fn chain<'a>(call: &'a CallExpression<'a>) -> Option<Chain<'a>> {
    let mut links = Vec::new();
    let mut current = call;
    let root = loop {
        let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&current.callee) else {
            return None;
        };
        if member.property.name != "append" || member.optional || current.optional {
            return None;
        }
        links.push(current);
        match unwrap_ts_wrappers(&member.object) {
            Expression::Identifier(id) => break id.name.to_string(),
            Expression::CallExpression(inner) => current = inner,
            _ => return None,
        }
    };
    links.reverse();
    (links.len() >= 2).then_some(Chain { root, links })
}

impl ScopeVisitor<'_> {
    /// Marks every link but the outermost as applied by the outer call.
    pub(super) fn consume_inner_links(&mut self, chain: &Chain<'_>) {
        for link in &chain.links[..chain.links.len() - 1] {
            self.consumed_links.insert(link.span.start);
        }
    }

    /// The alternatives of the root after each link, in order.
    pub(super) fn chain_variants(&self, chain: &Chain<'_>) -> Option<Vec<Recovered>> {
        let mut current = self.lookup(&chain.root)?.variants;
        for link in &chain.links {
            current = current.and_then(|prefix| self.append_step(&chain.root, prefix, link));
        }
        current
    }

    /// Applies each link's text to the root's recovered SQL.
    pub(super) fn apply_chain(&mut self, chain: &Chain<'_>) {
        for link in &chain.links {
            resolve::apply_append_to(self, &chain.root, link);
        }
    }
}
