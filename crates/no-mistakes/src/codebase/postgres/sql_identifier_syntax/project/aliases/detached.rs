use super::super::*;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;

mod forms;
mod obligations;
mod scope;
use forms::{Collector, Form, Kind, Node, Value};

pub(in crate::codebase::postgres::sql_identifier_syntax::project) fn collect(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    profiles: &[EmbeddedSqlOptions],
    selections: &mut [Selection],
    syntax: &mut Syntax<'_>,
) {
    if !profiles
        .iter()
        .any(|profile| scope::eligible(profile, module))
    {
        return;
    }
    let mut collector = Collector::new(module, profiles);
    collector.visit_program(program);
    for (index, (profile, selection)) in profiles.iter().zip(selections).enumerate() {
        if profile.import_specifier.is_empty() || profile.executor_names.is_empty() {
            continue;
        }
        let mut values = scope::seeds(profile, module);
        let nodes = collector
            .nodes
            .iter()
            .filter(|node| node.profile == index)
            .collect::<Vec<_>>();
        solve(&nodes, &mut values);
        let mut allowed = FxHashSet::default();
        for node in &nodes {
            if values.contains_key(&node.id) {
                allowed.extend(node.uses.iter().copied());
            }
        }
        let mut calls = Vec::new();
        for call in &collector.calls {
            let (root, parts) = (call.root, &call.parts);
            let Some(id) = collector.owners.get(&root) else {
                continue;
            };
            let Some(value) = values.get(id) else {
                continue;
            };
            let invokes = match value.kind {
                Kind::Executor => parts.len() == 1,
                Kind::Receiver => parts.len() == 2 && profile.executor_names.contains(&parts[1]),
            };
            if !invokes {
                continue;
            }
            allowed.insert(root);
            if value.kind == Kind::Executor && collector.node_ids.contains(id) {
                calls.push((call.span, *id, call.exact));
            }
        }
        for binding in &module.bindings {
            if binding.references.iter().any(|usage| {
                usage.runtime && !allowed.contains(&(usage.span.start, usage.span.end))
            }) {
                if let Some(value) = values.get_mut(&binding.id) {
                    value.exact = false;
                }
            }
        }
        propagate_uncertainty(&nodes, &mut values);
        for node in nodes.iter().filter(|node| values.contains_key(&node.id)) {
            if values[&node.id].exact {
                selection
                    .detached_initializers
                    .extend(node.uses.iter().copied());
            } else if values[&node.id].kind == Kind::Executor {
                selection.unsupported.insert(node.id);
            }
        }
        obligations::collect(profile, module, &collector, &nodes, &values, selection);
        selection
            .detached_executors
            .extend(values.iter().filter_map(|(id, value)| {
                (collector.node_ids.contains(id) && value.kind == Kind::Executor && value.exact)
                    .then_some(*id)
            }));
        for (at, id, exact) in calls {
            selection
                .detached_calls
                .insert(at, exact && values[&id].exact);
            syntax.detached_calls.insert(at);
        }
    }
}

fn propagate_uncertainty(nodes: &[&Node], values: &mut FxHashMap<usize, Value>) {
    loop {
        let mut changed = false;
        let invalid_receivers = scope::uncertain_receivers(nodes, values);
        for value in values.values_mut() {
            if value.exact && invalid_receivers.contains(&value.origin) {
                value.exact = false;
                changed = true;
            }
        }
        for node in nodes {
            let Some(previous) = values.get(&node.id).copied() else {
                continue;
            };
            if previous.exact && evaluate(node, values).is_none_or(|value| !value.exact) {
                values.get_mut(&node.id).unwrap().exact = false;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
}

fn solve(nodes: &[&Node], values: &mut FxHashMap<usize, Value>) {
    loop {
        let mut changed = false;
        for node in nodes {
            if let Some(value) = evaluate(node, values) {
                changed |= merge(node.id, value, values);
            }
        }
        if !changed {
            break;
        }
    }
}

fn merge(id: usize, mut candidate: Value, values: &mut FxHashMap<usize, Value>) -> bool {
    let Some(previous) = values.get(&id) else {
        values.insert(id, candidate);
        return true;
    };
    if previous.kind == Kind::Receiver && candidate.kind == Kind::Executor {
        candidate.exact = false;
        values.insert(id, candidate);
        return true;
    }
    if previous.exact && !candidate.exact {
        values.get_mut(&id).unwrap().exact = false;
        return true;
    }
    false
}

fn evaluate(node: &Node, values: &FxHashMap<usize, Value>) -> Option<Value> {
    let source = *values.get(&node.source)?;
    let (kind, valid) = match node.form {
        Form::Copy => (source.kind, true),
        Form::Query => (Kind::Executor, source.kind == Kind::Receiver),
        Form::Bind(receiver, query) => {
            let expected = if query {
                Kind::Receiver
            } else {
                Kind::Executor
            };
            if source.kind != expected {
                return None;
            }
            (
                Kind::Executor,
                receiver.and_then(|id| values.get(&id)).is_some_and(|this| {
                    this.kind == Kind::Receiver && this.origin == source.origin && this.exact
                }),
            )
        }
    };
    if matches!(node.form, Form::Query) && source.kind != Kind::Receiver {
        return None;
    }
    Some(Value {
        kind,
        origin: source.origin,
        exact: source.exact && node.exact && valid,
    })
}
