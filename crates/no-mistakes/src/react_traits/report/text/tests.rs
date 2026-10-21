use super::{
    callsite_props, print_results, print_results_md, print_usages, print_usages_md,
    print_violations, print_violations_md, render_results, render_results_md, render_usages,
    render_usages_md, render_violations, render_violations_md, target_label,
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
    let rendered = render_results(std::slice::from_ref(&facts), 0);
    assert_eq!(
        rendered,
        "\
app/components/Greeting.tsx#default
  hasState: false
  hasProps: false
  passesProps: false
  usesMemo: false
  usesContextProvider: false
  usesSuspense: false
  hasFetch: false
  environment: unknown

"
    );
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
    let rendered = render_results(std::slice::from_ref(&facts), 1);
    assert!(
        rendered.contains("  children:\n    app/components/Child.tsx#default\n"),
        "{rendered}"
    );
    assert!(
        rendered.contains("  dependencies:\n    app/components/Child.tsx\n"),
        "{rendered}"
    );
    print_results(&[facts], 1);
}

#[test]
fn print_results_with_inherited_from_children() {
    let mut facts = make_facts("default", "app/components/Parent.tsx");
    facts.inherited_from_children = Some(AggregatedFacts {
        has_fetch: true,
        ..Default::default()
    });
    let rendered = render_results(std::slice::from_ref(&facts), 0);
    assert!(
        rendered.contains("  inheritedFromChildren:\n    hasFetch: true\n"),
        "{rendered}"
    );
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
    let facts = [server, client, shared];
    let rendered = render_results(&facts, 0);
    assert!(rendered.contains("app/server.tsx#default\n"), "{rendered}");
    assert!(rendered.contains("environment: server\n"), "{rendered}");
    assert!(rendered.contains("environment: client\n"), "{rendered}");
    assert!(rendered.contains("environment: shared\n"), "{rendered}");
    print_results(&facts, 0);
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
    let rendered = render_results(std::slice::from_ref(&facts), 0);
    assert!(rendered.contains("hasFetch: true\n"), "{rendered}");
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
    assert_eq!(
        render_violations(&violations),
        "app/components/Fetcher.tsx#Fetcher: assert-no-fetch GET /api/users\n"
    );
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
    assert_eq!(
        render_violations(&violations),
        "app/components/Fetcher.tsx#Fetcher: assert-no-fetch\n"
    );
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
    assert_eq!(
        render_usages(&with_symbol),
        "\
app/Widget.tsx#Widget
  callsites:
    app/page.tsx:12 <Widget> [title, ...]
  stories:
    app/Widget.stories.tsx
  tests:
    app/Widget.test.tsx
  propTypes:
    WidgetProps
"
    );
    assert_eq!(
        render_usages_md(&with_symbol),
        "\
# React usages: `app/Widget.tsx#Widget`
## Callsites
- `app/page.tsx:12` `<Widget>` (title, ...)
## Stories
- `app/Widget.stories.tsx`
## Tests
- `app/Widget.test.tsx`
## Prop types
- `WidgetProps`
"
    );
    let widget = make_facts("Widget", "app/Widget.tsx");
    assert_eq!(
        render_results_md(std::slice::from_ref(&widget)),
        "# React traits\n- `app/Widget.tsx` `Widget` (unknown)\n"
    );
    let violation = Violation {
        component: "Widget".into(),
        file: "app/Widget.tsx".into(),
        rule: "assert-no-fetch".into(),
        detail: None,
    };
    assert_eq!(
        render_violations_md(std::slice::from_ref(&violation)),
        "# React trait violations\n- `app/Widget.tsx` `Widget`: assert-no-fetch\n"
    );
    print_usages(&with_symbol);
    print_usages_md(&with_symbol);
    print_results_md(std::slice::from_ref(&widget));
    print_violations_md(std::slice::from_ref(&violation));

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
    let plain = render_usages(&file_only);
    assert_eq!(
        plain,
        "\
app/Widget.tsx
  callsites:
    app/page.tsx:4 <default> []
"
    );
    assert!(
        !plain.contains("stories:") && !plain.contains("tests:") && !plain.contains("propTypes:"),
        "{plain}"
    );
    let plain_md = render_usages_md(&file_only);
    assert_eq!(
        plain_md,
        "\
# React usages: `app/Widget.tsx`
## Callsites
- `app/page.tsx:4` `<default>` ()
"
    );
    assert!(
        !plain_md.contains("## Stories")
            && !plain_md.contains("## Tests")
            && !plain_md.contains("## Prop types"),
        "{plain_md}"
    );
    print_usages(&file_only);
    print_usages_md(&file_only);
}
