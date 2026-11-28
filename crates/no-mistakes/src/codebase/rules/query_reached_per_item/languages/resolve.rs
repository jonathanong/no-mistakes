use super::regions::{split_separator, Scan};
use super::{style_for, Ctx, Sink};
use crate::codebase::rules::RuleFinding;
use std::collections::HashMap;

mod emit;
use emit::{proven, suppressed, unproven};

pub(super) fn project(
    file: &str,
    text: Option<&str>,
    extension: &str,
    ctx: &Ctx<'_>,
) -> Vec<RuleFinding> {
    let Some(source) = text else {
        return unreadable(file, ctx);
    };
    if style_for(extension).is_none() {
        return Vec::new();
    }
    project_scan(file, &super::regions::scan(source, extension), ctx)
}

fn project_scan(file: &str, scan: &Scan, ctx: &Ctx<'_>) -> Vec<RuleFinding> {
    let functions = index_functions(scan);
    let mut findings = Vec::new();
    for call in &scan.loops {
        let Some(outcome) = classify_loop(
            &call.callee,
            &call.function,
            &functions,
            &scan.shadows,
            ctx.sinks,
        ) else {
            continue;
        };
        if suppressed(ctx, file, &call.callee, call.line, &call.function) {
            continue;
        }
        match outcome {
            Outcome::Proven { kind, path } => findings.push(proven(
                file,
                call.line,
                &call.callee,
                call.construct,
                &kind,
                &path,
                ctx.message,
            )),
            Outcome::Unproven => {
                if let Some(finding) = unproven(file, call.line, &call.callee, call.construct, ctx)
                {
                    findings.push(finding);
                }
            }
        }
    }
    findings
}

fn unreadable(file: &str, ctx: &Ctx<'_>) -> Vec<RuleFinding> {
    unproven(file, 1, file, "file", ctx).into_iter().collect()
}

enum Outcome {
    Proven { kind: String, path: String },
    Unproven,
}

fn classify_loop(
    callee: &str,
    function: &str,
    functions: &HashMap<String, Result<Vec<String>, ()>>,
    shadows: &[(String, Vec<String>)],
    sinks: &[Sink],
) -> Option<Outcome> {
    match kind_of(callee, sinks) {
        Kind::Sink { kind, export } => Some(Outcome::Proven {
            kind: kind.to_string(),
            path: format!("{function} → {export}"),
        }),
        Kind::Clean => None,
        Kind::Receiver => Some(Outcome::Unproven),
        Kind::Bare if shadowed(shadows, function, callee) => Some(Outcome::Unproven),
        Kind::Bare => match functions.get(callee) {
            Some(Ok(calls)) => match function_reach(calls, sinks) {
                Reach::Sink { kind, export } => Some(Outcome::Proven {
                    kind,
                    path: format!("{function} → {callee} → {export}"),
                }),
                Reach::Clean => None,
                Reach::Unproven => Some(Outcome::Unproven),
            },
            Some(Err(())) | None => Some(Outcome::Unproven),
        },
    }
}

fn shadowed(shadows: &[(String, Vec<String>)], function: &str, callee: &str) -> bool {
    let mut rows = shadows.iter().filter(|(name, _)| name == function);
    match (rows.next(), rows.next()) {
        (Some((_, names)), None) => names.iter().any(|name| name == "*" || name == callee),
        (Some(_), Some(_)) => true,
        (None, _) => false,
    }
}

enum Kind<'a> {
    Sink { kind: &'a str, export: &'a str },
    Clean,
    Bare,
    Receiver,
}

fn kind_of<'a>(callee: &'a str, sinks: &'a [Sink]) -> Kind<'a> {
    let Some((head, tail)) = split_separator(callee) else {
        return Kind::Bare;
    };
    if tail.is_empty() {
        return Kind::Receiver;
    }
    let head = normalize(head);
    if let Some(sink) = sinks
        .iter()
        .find(|sink| sink.module == head && sink.export == tail)
    {
        return Kind::Sink {
            kind: &sink.kind,
            export: &sink.export,
        };
    }
    if sinks.iter().any(|sink| sink.module == head) {
        Kind::Clean
    } else {
        Kind::Receiver
    }
}

enum Reach {
    Sink { kind: String, export: String },
    Clean,
    Unproven,
}

fn function_reach(calls: &[String], sinks: &[Sink]) -> Reach {
    let mut unproven = false;
    for callee in calls {
        match kind_of(callee, sinks) {
            Kind::Sink { kind, export } => {
                return Reach::Sink {
                    kind: kind.to_string(),
                    export: export.to_string(),
                };
            }
            Kind::Clean => {}
            Kind::Bare | Kind::Receiver => unproven = true,
        }
    }
    if unproven {
        Reach::Unproven
    } else {
        Reach::Clean
    }
}

fn index_functions(scan: &Scan) -> HashMap<String, Result<Vec<String>, ()>> {
    let mut counts = HashMap::<&str, usize>::new();
    for name in &scan.functions {
        *counts.entry(name.as_str()).or_insert(0) += 1;
    }
    let mut grouped: HashMap<&str, Vec<String>> = HashMap::new();
    for (name, callee) in &scan.body_calls {
        grouped
            .entry(name.as_str())
            .or_default()
            .push(callee.clone());
    }
    counts
        .into_iter()
        .map(|(name, count)| {
            let calls = if count == 1 {
                Ok(grouped.remove(name).unwrap_or_default())
            } else {
                Err(())
            };
            (name.to_string(), calls)
        })
        .collect()
}

fn normalize(head: &str) -> String {
    head.replace("::", ".").replace(['\\', '/'], ".")
}
