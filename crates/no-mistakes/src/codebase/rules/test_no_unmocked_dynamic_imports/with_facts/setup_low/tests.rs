use super::classify::Classified;
use super::{policy, REWRITE};
use crate::config::v2::schema::RuleDef;
use crate::config::v2::{EffectiveRuleAnalysis, Effort, NoMistakesConfig, Unanalyzable};
use oxc_ast_visit::Visit;
use std::path::Path;

fn classify(path: &Path, source: &str) -> anyhow::Result<Classified> {
    crate::ast::with_program(path, source, |program, source| {
        let mut scan = super::classify::Scan::new(source, program);
        scan.visit_program(program);
        Classified::from(scan.into_reachability())
    })
}

fn classified(source: &str) -> Classified {
    classify(Path::new("setup.mts"), source).unwrap()
}

fn targets(source: &str) -> Vec<&'static str> {
    classified(source)
        .rejected
        .into_iter()
        .map(|shape| shape.target)
        .collect()
}

fn rule(resolved: Option<EffectiveRuleAnalysis>, options: &str) -> RuleDef {
    RuleDef {
        rule: "test-no-unmocked-dynamic-imports".to_string(),
        options: serde_yaml::from_str(options).unwrap(),
        resolved_analysis: resolved,
        ..RuleDef::default()
    }
}

#[test]
fn rewrite_names_the_accepted_setup_forms() {
    assert!(REWRITE.contains("top-level static import"));
    assert!(REWRITE.contains("vi.mock"));
    assert!(REWRITE.contains("jest.mock"));
    assert!(REWRITE.contains("literal require()"));
}

#[test]
fn accepts_static_import_hoisted_mocks_and_top_level_literal_requires() {
    let found = classified(
        r#"
        import "./static.mts";
        export const loaded = require("./required.cjs");
        require("./expr.cjs");
        const alias = require(`./template.cjs`);
        export default require("./default.cjs");
        vi.mock("./direct.mts");
        function nested() { jest.mock("./nested.mts"); }
        vi.mock(import("./typed.mts"));
        require.resolve("./resolved.cjs");
        "#,
    );
    assert_eq!(
        found.accepted_requires,
        vec![
            "./required.cjs",
            "./expr.cjs",
            "./template.cjs",
            "./default.cjs",
        ]
    );
    assert_eq!(
        found.accepted_mocks,
        vec!["./direct.mts", "./nested.mts", "./typed.mts"]
    );
    assert!(found.rejected.is_empty(), "{:?}", found.rejected);
}

#[test]
fn reports_other_setup_reachability_shapes_once() {
    let source = r#"
        true && require("./conditional.cjs");
        void require("./void.cjs");
        require("./seq-a.cjs"),
        require("./seq-b.cjs");
        if (require("./if.cjs")) {}
        function hidden() { require("./nested.cjs"); }
        import("./dynamic.mts");
        vi.doMock("./do.mts");
        vi.doMock(import("./do-typed.mts"));
        jest.setMock("./set.mts");
        jest.unstable_mockModule("./unstable.mts");
        require(name);
        "#;
    assert_eq!(
        targets(source),
        vec![
            "require",
            "require",
            "require",
            "require",
            "require",
            "require",
            "import",
            "doMock",
            "doMock",
            "setMock",
            "unstable_mockModule",
            "require",
        ]
    );
    let found = classified(source);
    assert!(found.accepted_requires.is_empty());
    assert!(found.accepted_mocks.is_empty());
    let typed = found
        .rejected
        .iter()
        .filter(|shape| shape.target == "doMock")
        .count();
    assert_eq!(typed, 2, "typed doMock must not also report its import()");
}

#[test]
fn non_literal_mock_calls_are_accepted_and_not_credited() {
    // A vi.mock/jest.mock call is accepted even when its specifier is not a
    // literal. The typed import() is that call's carrier, not a second shape.
    let found = classified(
        r#"
        const name = "./name.mts";
        vi.mock(name);
        vi.mock(import(name));
        jest.mock(name);
        "#,
    );
    assert!(found.rejected.is_empty(), "{:?}", found.rejected);
    assert!(
        found.accepted_mocks.is_empty(),
        "{:?}",
        found.accepted_mocks
    );
    assert!(found.accepted_requires.is_empty());
}

#[test]
fn unprepared_low_option_stays_on_max() {
    let mut config = NoMistakesConfig::default();
    config
        .rules
        .push(rule(None, "effort: low\nunanalyzable: ignore\n"));
    let resolved = policy(&config).unwrap();
    assert!(!resolved.low);
}

#[test]
fn mixed_effort_stays_on_max_and_all_low_uses_the_strictest_policy() {
    let low = |policy| {
        Some(EffectiveRuleAnalysis {
            effort: Effort::Low,
            unanalyzable: policy,
        })
    };
    let mut mixed = NoMistakesConfig::default();
    mixed.rules.push(rule(low(Unanalyzable::Ignore), "{}"));
    mixed.rules.push(rule(
        Some(EffectiveRuleAnalysis {
            effort: Effort::Max,
            unanalyzable: Unanalyzable::Report,
        }),
        "{}",
    ));
    assert!(!policy(&mixed).unwrap().low);

    let mut all_low = NoMistakesConfig::default();
    all_low.rules.push(rule(low(Unanalyzable::Ignore), "{}"));
    all_low.rules.push(rule(low(Unanalyzable::Advise), "{}"));
    all_low.rules.push(rule(low(Unanalyzable::Report), "{}"));
    let resolved = policy(&all_low).unwrap();
    assert!(resolved.low);
    assert_eq!(resolved.unanalyzable, Unanalyzable::Report);
}
