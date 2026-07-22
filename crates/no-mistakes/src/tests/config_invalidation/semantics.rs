use super::super::TestFramework;
use no_mistakes::config::v2::schema::{
    EffectKindConfig, FilesystemConfig, InfraConfig, NoMistakesConfig, PlaywrightTestConfig,
    Project, ProjectType, QueueConfig, QueuesTopLevelConfig, ReactTraitsConfig, RewriteRule,
    RuleDef, TestPlanFrameworkConfig, VitestConfig,
};
use std::collections::{BTreeMap, BTreeSet};

const DYNAMIC_IMPORT_RULE: &str = "test-no-unmocked-dynamic-imports";

pub(super) fn framework_semantics_changed(
    before: &NoMistakesConfig,
    after: &NoMistakesConfig,
    framework: TestFramework,
) -> bool {
    framework_test_plan(before, framework) != framework_test_plan(after, framework)
        || framework_tests(before, framework) != framework_tests(after, framework)
        || framework_trigger_projects(before, framework)
            != framework_trigger_projects(after, framework)
        || global_graph_semantics(before) != global_graph_semantics(after)
}

fn framework_test_plan(
    config: &NoMistakesConfig,
    framework: TestFramework,
) -> TestPlanFrameworkConfig {
    let mut plan = match framework {
        TestFramework::Dotnet => config.test_plan.dotnet.clone(),
        TestFramework::Playwright => config.test_plan.playwright.clone(),
        TestFramework::Vitest => config.test_plan.vitest.clone(),
        TestFramework::Swift => config.test_plan.swift.clone(),
    };
    // Compatibility warning bookkeeping is not behavior.
    plan.deprecated_dependencies_key = false;
    plan
}

fn framework_tests(config: &NoMistakesConfig, framework: TestFramework) -> serde_json::Value {
    match framework {
        TestFramework::Dotnet => serde_json::to_value(&config.tests.dotnet),
        TestFramework::Playwright => serde_json::to_value(&config.tests.playwright),
        // Vitest discovery reserves Playwright-owned paths before it applies
        // fallback matching, so ownership config changes require a fresh plan.
        TestFramework::Vitest => {
            serde_json::to_value((&config.tests.vitest, playwright_discovery_ownership(config)))
        }
        TestFramework::Swift => serde_json::to_value(&config.tests.swift),
    }
    .expect("test configuration must serialize")
}

fn playwright_discovery_ownership(config: &NoMistakesConfig) -> serde_json::Value {
    serde_json::json!({
        "configs": &config.tests.playwright.configs,
        "projects": config.tests.playwright.projects.iter().filter(|(_, policy)| {
            !policy.include.is_empty()
        }).map(|(name, policy)| {
            serde_json::json!({
                "name": name,
                "include": &policy.include,
                "exclude": &policy.exclude,
            })
        }).collect::<Vec<_>>(),
    })
}

fn framework_trigger_projects(
    config: &NoMistakesConfig,
    framework: TestFramework,
) -> Vec<(&str, Option<&Project>)> {
    let plan = match framework {
        TestFramework::Dotnet => &config.test_plan.dotnet,
        TestFramework::Playwright => &config.test_plan.playwright,
        TestFramework::Vitest => &config.test_plan.vitest,
        TestFramework::Swift => &config.test_plan.swift,
    };
    plan.full_suite_triggers
        .projects
        .keys()
        .map(|name| (name.as_str(), config.projects.get(name)))
        .collect()
}

/// Settings which affect the graph shared by every framework. A change here
/// cannot safely be attributed to one runner, so every framework fails open.
/// Resource-only project roots remain scoped through
/// `framework_trigger_projects`; roots that resolve routes or Next.js layout
/// ownership are retained below because they change shared graph semantics.
#[derive(PartialEq)]
struct GlobalGraphSemantics {
    frontend_root: Option<String>,
    assert_no_fetch: Option<bool>,
    react_traits: Option<ReactTraitsConfig>,
    filesystem: FilesystemConfig,
    infra: InfraConfig,
    queues: QueuesTopLevelConfig,
    effects: BTreeMap<String, EffectKindConfig>,
    projects: BTreeMap<String, GraphProjectSemantics>,
    // Only these rule options are read by graph configuration preparation.
    rules: Vec<RuleDef>,
    dynamic_runner_tests: DynamicRunnerTests,
}

#[derive(PartialEq)]
struct GraphProjectSemantics {
    type_: Option<ProjectType>,
    root: Option<String>,
    include: Vec<String>,
    exclude: Vec<String>,
    routes: Vec<String>,
    queues: QueueConfig,
    rewrites: Vec<RewriteRule>,
}

#[derive(Default, PartialEq)]
struct DynamicRunnerTests {
    // The dynamic-import filter's rule-target resolver supports only these
    // two runner target families.
    vitest: Option<VitestConfig>,
    playwright: Option<PlaywrightTestConfig>,
}

fn global_graph_semantics(config: &NoMistakesConfig) -> GlobalGraphSemantics {
    let dynamic_rule_projects = dynamic_rule_projects(config);
    let projects = config
        .projects
        .iter()
        .map(|(name, project)| {
            let dynamic_rule_target = dynamic_rule_projects.contains(name.as_str());
            (
                name.clone(),
                GraphProjectSemantics {
                    type_: project.type_.clone(),
                    root: graph_project_root(project, dynamic_rule_target),
                    include: if dynamic_rule_target {
                        project.include.clone()
                    } else {
                        Vec::new()
                    },
                    exclude: if dynamic_rule_target {
                        project.exclude.clone()
                    } else {
                        Vec::new()
                    },
                    routes: project.routes.clone(),
                    queues: project.queues.clone(),
                    rewrites: project.rewrites.clone(),
                },
            )
        })
        .collect();
    GlobalGraphSemantics {
        frontend_root: config.frontend_root.clone(),
        assert_no_fetch: config.assert_no_fetch,
        react_traits: config.react_traits.clone(),
        filesystem: config.filesystem.clone(),
        infra: config.infra.clone(),
        queues: config.queues.clone(),
        effects: config.effects.clone(),
        projects,
        rules: graph_rules(config),
        dynamic_runner_tests: dynamic_runner_tests(config),
    }
}

fn dynamic_runner_tests(config: &NoMistakesConfig) -> DynamicRunnerTests {
    let mut result = DynamicRunnerTests::default();
    for rule in config
        .rules
        .iter()
        .filter(|rule| rule.enabled && rule.rule == DYNAMIC_IMPORT_RULE)
    {
        if !rule.tests.vitest.is_empty() {
            result.vitest = Some(config.tests.vitest.clone());
        }
        if !rule.tests.playwright.is_empty() {
            result.playwright = Some(config.tests.playwright.clone());
        }
    }
    result
}

fn graph_project_root(project: &Project, dynamic_rule_target: bool) -> Option<String> {
    let server_routes =
        !project.routes.is_empty() && matches!(project.type_, None | Some(ProjectType::Server));
    (server_routes || project.type_ == Some(ProjectType::Nextjs) || dynamic_rule_target)
        .then(|| project.root.clone())
        .flatten()
}

fn dynamic_rule_projects(config: &NoMistakesConfig) -> BTreeSet<&str> {
    config
        .rules
        .iter()
        .filter(|rule| rule.enabled && rule.rule == DYNAMIC_IMPORT_RULE)
        .flat_map(|rule| rule.projects.iter())
        .filter(|name| config.projects.contains_key(name.as_str()))
        .map(String::as_str)
        .collect()
}

fn graph_rules(config: &NoMistakesConfig) -> Vec<RuleDef> {
    const GRAPH_RULES: [&str; 5] = [
        "route-consistency",
        "queue-dashboard-reachability",
        "http-route-static-paths",
        "http-call-static-paths",
        DYNAMIC_IMPORT_RULE,
    ];
    config
        .rules
        .iter()
        .filter(|rule| GRAPH_RULES.contains(&rule.rule.as_str()))
        .cloned()
        .collect()
}
