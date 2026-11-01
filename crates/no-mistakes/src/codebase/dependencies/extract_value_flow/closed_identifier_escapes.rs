use super::names::identifier_name;
use oxc_ast::ast::{CallExpression, Expression};

const READ_ONLY_METHODS: &[&str] = &[
    "at",
    "concat",
    "entries",
    "every",
    "filter",
    "find",
    "findIndex",
    "flat",
    "flatMap",
    "forEach",
    "get",
    "has",
    "includes",
    "indexOf",
    "join",
    "keys",
    "map",
    "reduce",
    "reduceRight",
    "slice",
    "some",
    "toString",
    "valueOf",
    "values",
];

const READ_ONLY_FUNCTIONS: &[&str] = &[
    "Object.entries",
    "Object.freeze",
    "Object.getOwnPropertyDescriptor",
    "Object.getOwnPropertyNames",
    "Object.getOwnPropertySymbols",
    "Object.is",
    "Object.isFrozen",
    "Object.keys",
    "Object.values",
];

impl super::super::super::ImportCollector {
    pub(in super::super::super) fn note_registry_alias(&mut self, init: &Expression<'_>) {
        let Some(name) = identifier_name(init) else {
            return;
        };
        self.record_registry_mutation(&name);
    }

    pub(in super::super::super) fn note_registry_argument_escape(
        &mut self,
        call: &CallExpression<'_>,
    ) {
        if read_only_call(call) {
            return;
        }
        for argument in &call.arguments {
            let Some(expression) = argument.as_expression() else {
                continue;
            };
            let Some(name) = identifier_name(expression) else {
                continue;
            };
            self.record_registry_mutation(&name);
        }
    }

    pub(in super::super::super) fn note_returned_registry(&mut self, argument: &Expression<'_>) {
        let Some(name) = identifier_name(argument) else {
            return;
        };
        self.record_registry_mutation(&name);
    }

    pub(in super::super::super) fn note_exported_registry(&mut self, name: &str) {
        if !self.collect_value_flow {
            return;
        }
        self.record_registry_mutation(name);
    }
}

fn read_only_call(call: &CallExpression<'_>) -> bool {
    super::super::super::simple_callee_name(&call.callee)
        .is_some_and(|name| READ_ONLY_FUNCTIONS.contains(&name.as_str()))
        || read_only_method(call)
}

fn read_only_method(call: &CallExpression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee)
    else {
        return false;
    };
    READ_ONLY_METHODS.contains(&member.property.name.as_str())
}
