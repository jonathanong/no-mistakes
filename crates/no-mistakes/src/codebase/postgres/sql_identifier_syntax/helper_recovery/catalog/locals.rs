//! Admit local constants only when every runtime use is a closed SQL read.
use super::{Lowered, PreparedCatalog};
use crate::codebase::postgres::{
    embedded::helpers::program::{Expr, Stmt},
    sql_identifier_syntax::Span,
};
use std::path::Path;

impl PreparedCatalog<'_> {
    pub(super) fn local_model(&self, file: &Path, id: usize, site: Span) -> Option<&Lowered> {
        self.valid_local(file, id, site, &mut Vec::new())?;
        Some(
            &self
                .files
                .get(file)?
                .sql_identifier_syntax
                .recovery
                .locals
                .get(&id)?
                .lowered,
        )
    }

    fn valid_local(
        &self,
        file: &Path,
        id: usize,
        site: Span,
        stack: &mut Vec<usize>,
    ) -> Option<()> {
        if stack.len() >= 8 || stack.contains(&id) {
            return None;
        }
        stack.push(id);
        let result = self.check_local(file, id, site, stack);
        stack.pop();
        result
    }

    fn check_local(
        &self,
        file: &Path,
        id: usize,
        site: Span,
        stack: &mut Vec<usize>,
    ) -> Option<()> {
        let module = self.module(file)?;
        let models = &self.files.get(file)?.sql_identifier_syntax.recovery;
        if models.dynamic_scope {
            return None;
        }
        let local = models.locals.get(&id)?;
        // A dependent const may be declared after this sink. Its later uses
        // still need to be closed, but only the queried const must exist now.
        if models.writes.contains(&id) || (stack.len() == 1 && local.declaration.1 > site.0) {
            return None;
        }
        let binding = module
            .bindings
            .iter()
            .find(|binding| binding.id == id && binding.runtime)?;
        if binding.scope_id == 0
            && module.exports.iter().any(|export| {
                !export.type_only && export.specifier.is_none() && export.local == binding.name
            })
        {
            return None;
        }
        let slot = format!("binding:{id}");
        for reference in binding
            .references
            .iter()
            .filter(|reference| reference.runtime)
        {
            let at = (reference.span.start, reference.span.end);
            if let Some(owner) = models.argument_owners.get(&at) {
                let call = &models.calls[*owner];
                let sink = self
                    .profile
                    .and_then(|profile| models.sinks.get(profile))
                    .is_some_and(|spans| spans.contains(&call.span));
                // A different executor invocation could retain or mutate a
                // builder/object received from this local before this sink.
                if sink
                    && call.span == site
                    && call
                        .arguments
                        .iter()
                        .enumerate()
                        .filter(|(_, argument)| {
                            argument.references.iter().any(|(used, _)| *used == id)
                        })
                        .all(|(index, argument)| {
                            if index == 0 {
                                use_shape(argument, &slot)
                            } else {
                                bind_reads(argument, &slot)
                            }
                        })
                {
                    continue;
                }
            }
            let dependent = models
                .locals
                .iter()
                .find(|(other, candidate)| {
                    **other != id
                        && candidate.initializer.0 <= at.0
                        && at.1 <= candidate.initializer.1
                        && use_shape(&candidate.lowered, &slot)
                })
                .map(|(other, _)| *other)?;
            self.valid_local(file, dependent, site, stack)?;
        }
        Some(())
    }
}

fn use_shape(lowered: &Lowered, slot: &str) -> bool {
    let [Stmt::Return(Some(expr), _)] = lowered.program.body.as_slice() else {
        return false;
    };
    match expr {
        Expr::Ident(name) => name == slot,
        Expr::Member(root, _) | Expr::ComputedMember(root, _) => {
            matches!(root.as_ref(), Expr::Ident(name) if name == slot)
        }
        // Raw templates can splice finite own-property text. Calls and whole
        // objects in holes remain escapes rather than read-only map accesses.
        Expr::Template(_, holes) => holes
            .iter()
            .all(|(expr, _)| template_read(expr, slot, true)),
        _ => false,
    }
}

fn template_read(expr: &Expr, slot: &str, computed: bool) -> bool {
    match expr {
        Expr::Ident(name) => name != slot,
        Expr::Lit(_) | Expr::Bool(_) | Expr::Null | Expr::Undefined | Expr::Num => true,
        Expr::Member(root, _) => matches!(root.as_ref(), Expr::Ident(name) if name == slot),
        Expr::ComputedMember(root, key) if computed => {
            matches!(root.as_ref(), Expr::Ident(name) if name == slot)
                && template_read(key, slot, computed)
        }
        Expr::Template(_, holes) => holes
            .iter()
            .all(|(expr, _)| template_read(expr, slot, computed)),
        Expr::Array(values) => values
            .iter()
            .all(|value| template_read(value, slot, computed)),
        _ => false,
    }
}

fn bind_reads(lowered: &Lowered, slot: &str) -> bool {
    matches!(lowered.program.body.as_slice(), [Stmt::Return(Some(expr), _)]
        if template_read(expr, slot, false))
}

#[cfg(test)]
mod tests;
