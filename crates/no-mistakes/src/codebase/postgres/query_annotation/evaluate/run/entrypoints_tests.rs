use super::super::reference::collect;
use super::Functions;
use crate::codebase::postgres::query_annotation::{DeleteKey, Expr, Function, Step};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

fn function(start: u32) -> Expr {
    Expr::Function(Arc::new(Function {
        start,
        params: Vec::new(),
        body: Vec::new(),
        supported: true,
        asynchronous: false,
        arrow: false,
        self_name: None,
    }))
}
fn old(globals: &BTreeMap<String, Expr>, values: &[Expr]) -> Vec<u32> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for value in values {
        collect(globals, value, &mut BTreeSet::new(), &mut seen, &mut result);
    }
    result.into_iter().map(|function| function.start).collect()
}
fn lazy(globals: &BTreeMap<String, Expr>, values: &[Expr]) -> Vec<u32> {
    let mut functions = Functions::new(globals);
    let mut result = Vec::new();
    for value in values {
        functions.start(value);
        result.extend(functions.by_ref().map(|function| function.start));
        assert!(functions.stack.is_empty());
        assert!(functions.visiting.is_empty());
    }
    result
}

#[test]
fn all_collect_containers_wrappers_and_ignored_variants_match_frozen_walk() {
    // Do not traverse wrappers absent from historical collect, even when they
    // contain a function: these describe effects rather than returned values.
    let included = vec![
        Expr::Record(vec![function(1)], vec!["key".into()]),
        Expr::Object(vec![function(2)]),
        Expr::Array(vec![function(3)]),
        Expr::Container(vec![function(4)]),
        Expr::Sequence(vec![function(5)]),
        Expr::Alternatives(vec![function(6)]),
        Expr::Spread(Box::new(function(7))),
        Expr::Member(Box::new(function(8)), "key".into()),
        Expr::Index(Box::new(function(9)), 0),
        Expr::OpaqueCallback(Box::new(function(10))),
        Expr::Discard(Box::new(function(11))),
        Expr::ComputedMember(Box::new(function(12)), Box::new(function(13))),
        Expr::Append(Box::new(function(14)), Box::new(function(15))),
    ];
    let ignored = vec![
        Expr::Number(1),
        Expr::Boolean(true),
        Expr::Primitive,
        Expr::Unknown,
        Expr::Unsupported,
        Expr::Text("text".into()),
        Expr::Name("missing".into()),
        Expr::Template(vec![function(16)]),
        Expr::Tagged("tag".into(), vec![function(17)], vec![function(18)]),
        Expr::Await(Box::new(function(19))),
        Expr::Call {
            callee: Box::new(function(20)),
            args: vec![function(21)],
            start: 1,
            end: 2,
        },
        Expr::ModuleLoad("module".into()),
        Expr::Children(vec![function(22)]),
        Expr::Flow(vec![Step::Effect(function(32))]),
        Expr::Binary("+".into(), Box::new(function(23)), Box::new(function(24))),
        Expr::Opaque(vec![function(25)]),
        Expr::OptionalCall {
            call: Box::new(function(26)),
            children: vec![function(27)],
        },
        Expr::OpaqueWrite {
            children: vec![function(28)],
            targets: vec!["target".into()],
        },
        Expr::SlotWrite {
            receiver: Box::new(function(29)),
            index: 0,
            value: Box::new(function(30)),
        },
        Expr::Delete(vec![function(31)], DeleteKey::Dynamic),
    ];
    let values = [included, ignored].concat();
    let globals = BTreeMap::new();
    let expected: Vec<u32> = (1..=15).collect();
    assert_eq!(old(&globals, &values), expected);
    assert_eq!(lazy(&globals, &values), expected);
}

#[test]
fn alias_cycle_exit_timing_and_start_dedup_match_across_yields_and_globals() {
    let globals = BTreeMap::from([
        (
            "a".into(),
            Expr::Array(vec![function(10), Expr::Name("b".into()), function(30)]),
        ),
        (
            "b".into(),
            Expr::Sequence(vec![Expr::Name("a".into()), function(20)]),
        ),
        ("duplicate_start".into(), function(10)),
        ("self".into(), Expr::Name("self".into())),
    ]);
    let values = vec![
        Expr::Name("a".into()),
        Expr::Name("b".into()),
        Expr::Name("duplicate_start".into()),
        Expr::Name("self".into()),
        function(40),
    ];
    assert_eq!(old(&globals, &values), vec![10, 20, 30, 40]);
    assert_eq!(lazy(&globals, &values), old(&globals, &values));
}

#[test]
fn dense_zero_function_container_uses_depth_bounded_frames() {
    // Width must not become a pending child Vec: real exported registries can
    // contain thousands of data-only properties and no speculative entrypoint.
    let values = vec![Expr::Record(
        (0..4096)
            .map(|index| Expr::Array(vec![Expr::Number(index), Expr::Text("scalar".into())]))
            .collect(),
        Vec::new(),
    )];
    let globals = BTreeMap::new();
    assert!(old(&globals, &values).is_empty());
    assert!(lazy(&globals, &values).is_empty());
}

#[test]
fn wide_registry_keeps_only_depth_sized_pending_frames_across_yields() {
    let globals = BTreeMap::new();
    let value = Expr::Array(
        (0..4096)
            .map(|index| Expr::Array(vec![function(index)]))
            .collect(),
    );
    let mut functions = Functions::new(&globals);
    functions.start(&value);
    for expected in 0..4096 {
        assert_eq!(functions.next().unwrap().start, expected);
        assert!(
            functions.stack.len() <= 2,
            "width must not grow pending DFS frames"
        );
    }
    assert!(functions.next().is_none());
}
