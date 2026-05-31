fn local_call_graph(
    calls: &[crate::codebase::dependencies::extract::FunctionCall],
) -> HashMap<String, Vec<String>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for call in calls {
        if let Some(caller) = &call.caller {
            graph
                .entry(caller.clone())
                .or_default()
                .push(call.callee.clone());
        }
    }
    for callees in graph.values_mut() {
        callees.sort();
        callees.dedup();
    }
    graph
}

fn local_ordered_call_graph(
    calls: &[crate::codebase::dependencies::extract::FunctionCall],
) -> HashMap<String, Vec<String>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for call in calls {
        if let Some(caller) = &call.caller {
            graph
                .entry(caller.clone())
                .or_default()
                .push(call.callee.clone());
        }
    }
    graph
}

fn local_call_records(calls: &[FunctionCall]) -> HashMap<String, Vec<FunctionCall>> {
    let mut graph: HashMap<String, Vec<FunctionCall>> = HashMap::new();
    for call in calls {
        if let Some(caller) = &call.caller {
            graph.entry(caller.clone()).or_default().push(call.clone());
        }
    }
    for calls in graph.values_mut() {
        calls.sort_by(|left, right| {
            (&left.callee, &left.static_arg).cmp(&(&right.callee, &right.static_arg))
        });
        calls.dedup();
    }
    graph
}
