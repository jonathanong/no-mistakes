use super::super::super::Bindings;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{
    CallableId, ValueBinding, ValueCall, ValueExpression, ValueFlowFacts, ValueParameter,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{fx_map, fx_set};
use std::path::PathBuf;
use std::sync::Arc;

fn named(name: &str, offset: u32) -> ValueExpression {
    ValueExpression::Named {
        name: name.to_string(),
        scope: Some(0),
        offset,
        context: None,
    }
}

fn bind(name: &str, at: u32, value: ValueExpression) -> ValueBinding {
    ValueBinding {
        immutable: true,
        owner: None,
        name: name.to_string(),
        scope: 0,
        declared_at: at,
        repeated: false,
        invalidated_at: None,
        mutation_contexts: fx_set(),
        value,
    }
}

fn parameter(name: &str) -> ValueParameter {
    ValueParameter {
        owner: CallableId(1),
        index: 0,
        name: name.to_string(),
        scope: 0,
        default: ValueExpression::Undefined,
        projection: Vec::new(),
        initialization_order: 0,
        default_order: 0,
        default_mutations: None,
        invalidated_at: None,
        mutation_contexts: fx_set(),
        type_reference: None,
    }
}

fn call(offset: u32, end: u32, callee: ValueExpression) -> ValueCall {
    ValueCall {
        offset,
        end,
        source_callee: String::new(),
        context: None,
        callee,
        arguments: Arc::new(Vec::new()),
        last_argument_literal: false,
        bound: false,
        spread_at: None,
        once: false,
        repeated: false,
    }
}

fn object(field: &str, value: ValueExpression) -> ValueExpression {
    let mut fields = fx_map();
    fields.insert(field.to_string(), value);
    ValueExpression::Object(Arc::new(fields), false)
}

fn member(inner: ValueExpression, key: &str) -> ValueExpression {
    ValueExpression::Member(Arc::new(inner), key.to_string())
}

fn process_env(key: &str) -> ValueExpression {
    let root = ValueExpression::Named {
        name: "process".to_string(),
        scope: None,
        offset: 1,
        context: None,
    };
    member(member(root, "env"), key)
}

struct Harness {
    facts: CheckFactMap,
    file: PathBuf,
}

impl Harness {
    fn new(ts: TsFileFacts) -> Self {
        let file = PathBuf::from("query.ts");
        let mut facts = CheckFactMap::default();
        facts.files.push(file.clone());
        facts.ts.insert(
            file.clone(),
            Arc::new(CheckFileFacts {
                ts: Arc::new(ts),
                ..CheckFileFacts::default()
            }),
        );
        Self { facts, file }
    }

    fn unbounded(&self, name: &str) -> bool {
        Bindings::new(&self.facts).flows_unbounded(&self.file, 0, name, 100)
    }
}

fn flow() -> ValueFlowFacts {
    ValueFlowFacts {
        parameters: vec![parameter("free"), parameter("missing")],
        bindings: vec![
            bind("schema", 1, process_env("SCHEMA")),
            bind("alias", 2, named("free", 40)),
            bind(
                "waited",
                3,
                ValueExpression::Awaited(Arc::new(named("free", 40))),
            ),
            bind(
                "parts",
                4,
                ValueExpression::Derived(Arc::from([named("free", 40)])),
            ),
            bind(
                "either",
                5,
                ValueExpression::Either(
                    Arc::new(named("free", 40)),
                    Arc::new(ValueExpression::Unknown),
                ),
            ),
            bind(
                "nullish",
                6,
                ValueExpression::Nullish(
                    Arc::new(named("free", 40)),
                    Arc::new(ValueExpression::Null),
                ),
            ),
            bind(
                "fallback",
                7,
                ValueExpression::Default(
                    Arc::new(named("free", 40)),
                    Arc::new(ValueExpression::Undefined),
                ),
            ),
            bind("opaque", 8, ValueExpression::Unknown),
            bind("loop_a", 9, named("loop_b", 10)),
            bind("loop_b", 10, named("loop_a", 9)),
            bind(
                "from_object",
                11,
                member(object("table", named("free", 40)), "table"),
            ),
            bind("from_name", 12, member(named("held", 13), "table")),
            bind("held", 13, object("table", named("free", 40))),
            bind(
                "from_wait",
                14,
                member(
                    ValueExpression::Awaited(Arc::new(object("table", named("free", 40)))),
                    "table",
                ),
            ),
            bind(
                "from_either",
                15,
                member(
                    ValueExpression::Either(
                        Arc::new(object("table", named("free", 40))),
                        Arc::new(ValueExpression::Unknown),
                    ),
                    "table",
                ),
            ),
            bind("arrow", 16, ValueExpression::Callable(CallableId(7))),
            bind("declared", 17, ValueExpression::Unknown),
            bind(
                "from_arrow",
                18,
                member(ValueExpression::Call(20, 30), "table"),
            ),
            bind(
                "from_fn",
                19,
                member(ValueExpression::Call(40, 50), "table"),
            ),
            bind(
                "from_super",
                21,
                member(ValueExpression::Call(60, 70), "table"),
            ),
            bind(
                "from_await_call",
                22,
                member(ValueExpression::Call(80, 90), "table"),
            ),
            bind(
                "from_cycle_call",
                23,
                member(ValueExpression::Call(100, 110), "table"),
            ),
            bind(
                "from_named_return",
                24,
                member(ValueExpression::Call(120, 130), "table"),
            ),
            bind("box", 25, named("box", 25)),
            bind(
                "missing_call",
                26,
                member(ValueExpression::Call(1, 2), "table"),
            ),
            bind(
                "unknown_member",
                27,
                member(ValueExpression::Unknown, "table"),
            ),
        ],
        returns: vec![
            (CallableId(7), object("table", named("free", 40))),
            (CallableId(8), object("table", named("free", 40))),
            (CallableId(9), object("table", named("free", 40))),
            (
                CallableId(10),
                ValueExpression::Awaited(Arc::new(object("table", named("free", 40)))),
            ),
            (
                CallableId(11),
                ValueExpression::Derived(Arc::from([object("table", named("free", 40))])),
            ),
            (
                CallableId(12),
                ValueExpression::Either(
                    Arc::new(object("table", named("free", 40))),
                    Arc::new(ValueExpression::Null),
                ),
            ),
            (
                CallableId(13),
                ValueExpression::Nullish(
                    Arc::new(object("table", named("free", 40))),
                    Arc::new(ValueExpression::Undefined),
                ),
            ),
            (
                CallableId(14),
                ValueExpression::Default(
                    Arc::new(object("table", named("free", 40))),
                    Arc::new(ValueExpression::Unknown),
                ),
            ),
            (CallableId(15), named("boxed", 28)),
            (
                CallableId(16),
                object("table", member(ValueExpression::Call(100, 110), "table")),
            ),
        ],
        calls: vec![
            call(20, 30, ValueExpression::Callable(CallableId(7))),
            call(40, 50, named("declared", 17)),
            call(60, 70, ValueExpression::Super(CallableId(9))),
            call(
                80,
                90,
                ValueExpression::Awaited(Arc::new(ValueExpression::Callable(CallableId(10)))),
            ),
            call(100, 110, ValueExpression::Callable(CallableId(16))),
            call(120, 130, ValueExpression::Callable(CallableId(11))),
            call(140, 150, ValueExpression::Callable(CallableId(12))),
            call(160, 170, ValueExpression::Callable(CallableId(13))),
            call(180, 190, ValueExpression::Callable(CallableId(14))),
            call(200, 210, ValueExpression::Callable(CallableId(15))),
        ],
        ..ValueFlowFacts::default()
    }
}

fn facts(flow: ValueFlowFacts) -> TsFileFacts {
    TsFileFacts {
        callable_bindings: vec![(0, "declared".to_string(), CallableId(8))],
        value_flow: flow,
        ..TsFileFacts::default()
    }
}

#[test]
fn flows_unbounded_follows_parameter_containers_and_returns() {
    let harness = Harness::new(facts(flow()));
    let file = harness.file.clone();
    for name in [
        "free",
        "schema",
        "alias",
        "waited",
        "parts",
        "either",
        "nullish",
        "fallback",
        "from_object",
        "from_name",
        "from_wait",
        "from_either",
        "from_arrow",
        "from_fn",
        "from_super",
        "from_await_call",
        "from_named_return",
    ] {
        assert!(harness.unbounded(name), "{name}");
    }
    for name in [
        "opaque",
        "loop_a",
        "missing_call",
        "unknown_member",
        "absent",
        "from_cycle_call",
    ] {
        assert!(!harness.unbounded(name), "{name}");
    }
    let mut extra = flow();
    extra.returns.push((CallableId(17), named("box", 25)));
    extra
        .calls
        .push(call(220, 230, ValueExpression::Callable(CallableId(17))));
    extra
        .returns
        .push((CallableId(18), object("other", named("free", 40))));
    extra
        .calls
        .push(call(240, 250, ValueExpression::Callable(CallableId(18))));
    extra.bindings.push(bind(
        "from_either_return",
        29,
        member(ValueExpression::Call(140, 150), "table"),
    ));
    extra.bindings.push(bind(
        "from_nullish_return",
        30,
        member(ValueExpression::Call(160, 170), "table"),
    ));
    extra.bindings.push(bind(
        "from_default_return",
        31,
        member(ValueExpression::Call(180, 190), "table"),
    ));
    extra.bindings.push(bind(
        "from_named_field",
        32,
        member(ValueExpression::Call(200, 210), "table"),
    ));
    extra
        .bindings
        .push(bind("boxed", 28, object("table", named("free", 40))));
    extra.bindings.push(bind(
        "self_member",
        33,
        member(named("self_member", 33), "table"),
    ));
    extra.bindings.push(bind(
        "from_box_cycle",
        34,
        member(ValueExpression::Call(220, 230), "table"),
    ));
    extra.bindings.push(bind(
        "from_missing_field",
        35,
        member(ValueExpression::Call(240, 250), "table"),
    ));
    extra.bindings.push(bind(
        "from_nullish_member",
        36,
        member(
            ValueExpression::Nullish(
                Arc::new(ValueExpression::Unknown),
                Arc::new(object("table", named("free", 40))),
            ),
            "table",
        ),
    ));
    extra.bindings.push(bind(
        "from_default_member",
        37,
        member(
            ValueExpression::Default(
                Arc::new(ValueExpression::Unknown),
                Arc::new(object("table", named("free", 40))),
            ),
            "table",
        ),
    ));
    extra.returns.push((
        CallableId(19),
        ValueExpression::Either(
            Arc::new(ValueExpression::Unknown),
            Arc::new(object("table", named("free", 40))),
        ),
    ));
    extra
        .calls
        .push(call(260, 270, ValueExpression::Callable(CallableId(19))));
    extra.bindings.push(bind(
        "from_either_right",
        38,
        member(ValueExpression::Call(260, 270), "table"),
    ));
    extra.returns.push((
        CallableId(20),
        ValueExpression::Named {
            name: "no_such".to_string(),
            scope: Some(0),
            offset: 1,
            context: None,
        },
    ));
    extra
        .calls
        .push(call(280, 290, ValueExpression::Callable(CallableId(20))));
    extra.bindings.push(bind(
        "from_missing_name",
        39,
        member(ValueExpression::Call(280, 290), "table"),
    ));
    extra
        .bindings
        .push(bind("as_fn", 41, ValueExpression::Callable(CallableId(21))));
    extra
        .returns
        .push((CallableId(21), object("table", named("free", 40))));
    extra.calls.push(call(300, 310, named("as_fn", 41)));
    extra.bindings.push(bind(
        "from_named_callable",
        42,
        member(ValueExpression::Call(300, 310), "table"),
    ));
    extra.calls.push(call(320, 330, ValueExpression::Unknown));
    extra.bindings.push(bind(
        "from_unknown_callee",
        43,
        member(ValueExpression::Call(320, 330), "table"),
    ));
    let harness = Harness::new(facts(extra));
    assert_eq!(harness.file, file);
    for name in [
        "from_either_return",
        "from_nullish_return",
        "from_default_return",
        "from_named_field",
        "from_nullish_member",
        "from_default_member",
        "from_either_right",
        "from_named_callable",
    ] {
        assert!(harness.unbounded(name), "{name}");
    }
    assert!(!harness.unbounded("self_member"));
    assert!(!harness.unbounded("from_box_cycle"));
    assert!(!harness.unbounded("from_missing_field"));
    assert!(!harness.unbounded("from_missing_name"));
    assert!(!harness.unbounded("from_unknown_callee"));
}

#[test]
fn parameter_lookup_is_scoped_to_the_reference() {
    let mut other = parameter("table");
    other.scope = 7;
    let mut numeric = parameter("count");
    numeric.scope = 8;
    numeric.type_reference = Some(crate::codebase::dependencies::extract::ValueType::Number);
    let case = Harness::new(TsFileFacts {
        value_flow: ValueFlowFacts {
            parameters: vec![other, numeric],
            ..ValueFlowFacts::default()
        },
        ..TsFileFacts::default()
    });
    let bindings = Bindings::new(&case.facts);
    assert!(bindings.parameter(&case.file, 7, "table"));
    assert!(!bindings.parameter(&case.file, 9, "table"));
    assert!(bindings.parameter(&case.file, 0, "table"));
    assert!(!bindings.parameter(&case.file, 8, "count"));
    assert!(!bindings.parameter(&case.file, 7, "missing"));
    assert!(!bindings.closed_parameter(&case.file, 7, "table"));
    assert!(bindings.closed_parameter(&case.file, 8, "count"));
    assert!(!bindings.closed_parameter(&case.file, 0, "count"));
    assert!(!bindings.closed_parameter(&case.file, 8, "missing"));
}
