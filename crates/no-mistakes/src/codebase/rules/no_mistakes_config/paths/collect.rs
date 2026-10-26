use super::{Field, Kind, Ref};
use crate::config::v2::schema::{StringOrList, TestPlanFrameworkConfig};
use crate::config::v2::NoMistakesConfig;
mod rule_options;
mod test_paths;
mod test_plan_paths;

pub(super) fn collect(config: &NoMistakesConfig, refs: &mut Vec<Ref>, origins: bool) {
    push_opt(
        refs,
        Field::new("frontendRoot", origins),
        Kind::Directory,
        config.frontend_root.as_deref(),
    );
    for (name, project) in &config.projects {
        push_opt(
            refs,
            Field::new("projects", origins).key(name).key("root"),
            Kind::Directory,
            project.root.as_deref(),
        );
    }
    test_paths::collect(config, refs, origins);
    test_plan_paths::collect(config, refs, origins);
    rule_options::collect(config, refs, origins);
}

pub(super) fn path_kind(value: &str) -> Kind {
    if value.contains('*') || value.contains('?') || value.contains('{') {
        Kind::Glob
    } else {
        Kind::File
    }
}

pub(crate) fn frameworks(
    config: &NoMistakesConfig,
) -> [(&'static str, &TestPlanFrameworkConfig); 14] {
    [
        ("dotnet", &config.test_plan.dotnet),
        ("playwright", &config.test_plan.playwright),
        ("vitest", &config.test_plan.vitest),
        ("swift", &config.test_plan.swift),
        ("python", &config.test_plan.python),
        ("go", &config.test_plan.go),
        ("cargo", &config.test_plan.cargo),
        ("rails", &config.test_plan.rails),
        ("php", &config.test_plan.php),
        ("java", &config.test_plan.java),
        ("kotlin", &config.test_plan.kotlin),
        ("elixir", &config.test_plan.elixir),
        ("dart", &config.test_plan.dart),
        ("jest", &config.test_plan.jest),
    ]
}

fn push_list(refs: &mut Vec<Ref>, field: Field, kind: Kind, values: &Option<StringOrList>) {
    let Some(values) = values else {
        return;
    };
    for (index, value) in values.values().iter().enumerate() {
        push(refs, field.index(index), kind, value);
    }
}

fn push_opt(refs: &mut Vec<Ref>, field: Field, kind: Kind, value: Option<&str>) {
    if let Some(value) = value {
        push(refs, field, kind, value);
    }
}

pub(super) fn push(refs: &mut Vec<Ref>, field: Field, kind: Kind, value: &str) {
    if !value.is_empty() && !value.starts_with('!') {
        refs.push(Ref {
            field: field.label,
            origin: field.origin,
            rule_index: field.rule_index,
            kind,
            value: value.to_string(),
        });
    }
}
