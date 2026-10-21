use super::{
    callsite_props, print_results, print_results_md, print_usages, print_usages_md,
    print_violations, print_violations_md, target_label,
};
use crate::react_traits::report::types::{
    AggregatedFacts, Callsite, ComponentFacts, ComponentRef, Environment, FetchCall, UsagesReport,
    UsagesTarget, Violation,
};

fn make_facts(name: &str, file: &str) -> ComponentFacts {
    ComponentFacts {
        name: name.to_string(),
        file: file.to_string(),
        environment: Environment::Unknown,
        has_state: false,
        has_props: false,
        passes_props: false,
        uses_memo: false,
        uses_context_provider: false,
        uses_suspense: false,
        fetches: vec![],
        dependencies: vec![],
        children: vec![],
        inherited_from_children: None,
    }
}

#[test]
fn print_results_no_depth_produces_output() {
    let facts = make_facts("default", "app/components/Greeting.tsx");
    print_results(&[facts], 0);
}

#[test]
fn print_results_with_depth_produces_output() {
    let mut facts = make_facts("default", "app/components/Parent.tsx");
    facts.children = vec![ComponentRef {
        name: "default".to_string(),
        file: "app/components/Child.tsx".to_string(),
    }];
    facts.dependencies = vec!["app/components/Child.tsx".to_string()];
    print_results(&[facts], 1);
}

#[test]
fn print_results_with_inherited_from_children() {
    let mut facts = make_facts("default", "app/components/Parent.tsx");
    facts.inherited_from_children = Some(AggregatedFacts {
        has_fetch: true,
        ..Default::default()
    });
    print_results(&[facts], 0);
}

#[test]
fn print_results_with_various_environments() {
    let mut server = make_facts("default", "app/server.tsx");
    server.environment = Environment::Server;
    let mut client = make_facts("default", "app/client.tsx");
    client.environment = Environment::Client;
    let mut shared = make_facts("default", "app/shared.tsx");
    shared.environment = Environment::Shared;
    print_results(&[server, client, shared], 0);
}

#[test]
fn print_results_with_fetches() {
    let mut facts = make_facts("default", "app/components/Fetcher.tsx");
    facts.fetches = vec![FetchCall {
        file: "app/components/Fetcher.tsx".to_string(),
        exported_name: None,
        shape: Some("GET /api/users".to_string()),
        line: 1,
    }];
    print_results(&[facts], 0);
}

#[test]
fn print_violations_outputs_violations() {
    let violations = vec![Violation {
        component: "Fetcher".to_string(),
        file: "app/components/Fetcher.tsx".to_string(),
        rule: "assert-no-fetch".to_string(),
        detail: Some("GET /api/users".to_string()),
    }];
    print_violations(&violations);
}

#[test]
fn print_violations_no_detail() {
    let violations = vec![Violation {
        component: "Fetcher".to_string(),
        file: "app/components/Fetcher.tsx".to_string(),
        rule: "assert-no-fetch".to_string(),
        detail: None,
    }];
    print_violations(&violations);
}

#[test]
fn violation_json_shape_remains_public_four_fields() {
    let violation = Violation {
        component: "Fetcher".to_string(),
        file: "app/components/Fetcher.tsx".to_string(),
        rule: "assert-no-fetch".to_string(),
        detail: None,
    };

    assert_eq!(
        serde_json::to_value(violation).unwrap(),
        serde_json::json!({
            "component": "Fetcher",
            "file": "app/components/Fetcher.tsx",
            "rule": "assert-no-fetch",
            "detail": null,
        })
    );
}

#[test]
fn aggregated_facts_keeps_the_public_seven_field_literal_shape() {
    let _facts = AggregatedFacts {
        has_state: false,
        has_props: false,
        passes_props: false,
        uses_memo: false,
        uses_context_provider: false,
        uses_suspense: false,
        has_fetch: false,
    };
}

fn usages_report() -> UsagesReport {
    UsagesReport {
        target: UsagesTarget {
            file: "app/Widget.tsx".into(),
            symbol: Some("Widget".into()),
        },
        callsites: vec![Callsite {
            file: "app/page.tsx".into(),
            line: 12,
            component: "Widget".into(),
            props: vec!["title".into()],
            has_spread: true,
        }],
        stories: Some(vec!["app/Widget.stories.tsx".into()]),
        tests: Some(vec!["app/Widget.test.tsx".into()]),
        prop_types: Some(vec!["WidgetProps".into()]),
    }
}

#[test]
fn usages_labels_include_symbol_spread_and_optional_sections() {
    let with_symbol = usages_report();
    assert_eq!(target_label(&with_symbol), "app/Widget.tsx#Widget");
    assert_eq!(callsite_props(&with_symbol.callsites[0]), "title, ...");
    print_usages(&with_symbol);
    print_usages_md(&with_symbol);
    print_results_md(&[make_facts("Widget", "app/Widget.tsx")]);
    print_violations_md(&[Violation {
        component: "Widget".into(),
        file: "app/Widget.tsx".into(),
        rule: "assert-no-fetch".into(),
        detail: None,
    }]);

    let file_only = UsagesReport {
        target: UsagesTarget {
            file: "app/Widget.tsx".into(),
            symbol: None,
        },
        callsites: vec![Callsite {
            file: "app/page.tsx".into(),
            line: 4,
            component: "default".into(),
            props: vec![],
            has_spread: false,
        }],
        stories: None,
        tests: None,
        prop_types: None,
    };
    assert_eq!(target_label(&file_only), "app/Widget.tsx");
    assert_eq!(callsite_props(&file_only.callsites[0]), "");
    print_usages(&file_only);
    print_usages_md(&file_only);
}
