use super::document::Document;
use super::{DocumentFacts, Environment, Scope};
use serde_yaml::Value;

mod visit;

pub(super) fn collect(value: &Value, source: &str, action: bool) -> DocumentFacts {
    let Some(document) = Document::parse(source) else {
        return super::failed("source locations could not be prepared".into());
    };
    if !value.is_mapping() {
        return super::failed("document root must be a mapping".into());
    }
    let mut result = DocumentFacts::default();
    if action {
        result.composite = value
            .get("runs")
            .and_then(|runs| runs.get("using"))
            .and_then(Value::as_str)
            == Some("composite");
        if result.composite {
            visit::visit(
                value,
                Some(0),
                &document,
                false,
                visit::IfScope::Action,
                &mut result.references,
            );
        }
        return result;
    }
    let mapping = value.as_mapping().expect("mapping checked above");
    for (key, child) in mapping {
        let Some(key) = key.as_str() else {
            result
                .incomplete
                .push((1, "workflow keys must be strings".into()));
            continue;
        };
        if matches!(key, "on" | "jobs") {
            continue;
        }
        visit::visit(
            child,
            document.mapping_value(0, key),
            &document,
            false,
            visit::IfScope::Plain,
            &mut result.references,
        );
    }
    let jobs = value.get("jobs").and_then(Value::as_mapping);
    let job_node = document.mapping_value(0, "jobs");
    if value.get("jobs").is_some() && jobs.is_none() {
        result
            .incomplete
            .push((document.line(job_node), "jobs must be a mapping".into()));
    }
    for (key, job) in jobs.into_iter().flatten() {
        let Some(key) = key.as_str() else {
            result
                .incomplete
                .push((document.line(job_node), "job keys must be strings".into()));
            continue;
        };
        let node = job_node.and_then(|id| document.mapping_value(id, key));
        let mut scope = Scope {
            line: document.line(node),
            ..Default::default()
        };
        if !job.is_mapping() {
            result
                .incomplete
                .push((scope.line, "job must be a mapping".into()));
        }
        if job.get("uses").is_some()
            && job.get("secrets").is_some_and(|value| {
                value.as_str() != Some("inherit")
                    && !value.as_mapping().is_some_and(|values| {
                        values.iter().all(|(key, value)| {
                            key.is_string() && !value.is_mapping() && !value.is_sequence()
                        })
                    })
            })
        {
            let secrets = node.and_then(|id| document.mapping_value(id, "secrets"));
            result.incomplete.push((
                document.line(secrets),
                "reusable secret bindings are not a static mapping or inherit".into(),
            ));
        }
        scope.environment = environment(job.get("environment"));
        if matches!(scope.environment, Environment::Unknown) {
            let environment_node = node.and_then(|id| document.mapping_value(id, "environment"));
            result.incomplete.push((
                document.line(environment_node),
                "job environment is not a literal name".into(),
            ));
        }
        visit::visit(
            job,
            node,
            &document,
            false,
            visit::IfScope::Condition,
            &mut scope.references,
        );
        result.jobs.insert(key.to_owned(), scope);
    }
    result
}

fn environment(value: Option<&Value>) -> Environment {
    let Some(value) = value else {
        return Environment::Repository;
    };
    match value
        .as_str()
        .or_else(|| value.get("name").and_then(Value::as_str))
    {
        Some(name) if !name.is_empty() && !name.contains("${{") => {
            Environment::Named(name.to_owned())
        }
        _ => Environment::Unknown,
    }
}
