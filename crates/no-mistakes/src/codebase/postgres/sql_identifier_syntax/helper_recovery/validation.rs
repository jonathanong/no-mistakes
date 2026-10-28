use super::super::template_binds::TemplateBinds;
use super::FileModels;
use crate::codebase::postgres::embedded::helpers::program::Stmt;

impl FileModels {
    pub(in crate::codebase::postgres::sql_identifier_syntax) fn validate(
        &mut self,
        tags: &TemplateBinds,
    ) {
        self.locals
            .retain(|_, local| local.lowered.tags.iter().all(|id| tags.is_valid(*id)));
        self.functions
            .retain(|_, function| function.lowered.tags.iter().all(|id| tags.is_valid(*id)));
        // Retain raw callee identities for abrupt-effect admission even when
        // the argument value proof is invalidated by a later tag write.
        for call in &mut self.calls {
            for argument in &mut call.arguments {
                if argument.tags.iter().any(|id| !tags.is_valid(*id)) {
                    argument.program.body = vec![Stmt::Opaque];
                }
            }
        }
        self.calls.sort_by_key(|call| call.span);
        for function in self.functions.values() {
            if let Some(binding) = function.binding {
                self.binding_functions
                    .entry(binding)
                    .and_modify(|id| *id = None)
                    .or_insert(Some(function.id));
            }
        }
        for (index, call) in self.calls.iter().enumerate() {
            for argument in &call.arguments {
                for (_, at) in &argument.references {
                    self.argument_owners
                        .entry(*at)
                        .and_modify(|owner| {
                            let previous = &self.calls[*owner];
                            if call.span.1 - call.span.0 < previous.span.1 - previous.span.0 {
                                *owner = index;
                            }
                        })
                        .or_insert(index);
                }
            }
        }
    }
}
