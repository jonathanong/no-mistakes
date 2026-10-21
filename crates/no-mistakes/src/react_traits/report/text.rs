use crate::react_traits::report::types::{ComponentFacts, UsagesReport, Violation};
use std::fmt::Write as _;

pub fn print_results(results: &[ComponentFacts], depth: usize) {
    print!("{}", render_results(results, depth));
}

fn render_results(results: &[ComponentFacts], depth: usize) -> String {
    let mut out = String::new();
    for facts in results {
        render_component(&mut out, facts, depth, 0);
        let _ = writeln!(out);
    }
    out
}

fn render_component(
    out: &mut String,
    facts: &ComponentFacts,
    max_depth: usize,
    current_depth: usize,
) {
    let indent = "  ".repeat(current_depth);
    let _ = writeln!(out, "{}{}#{}", indent, facts.file, facts.name);
    let i2 = "  ".repeat(current_depth + 1);
    let _ = writeln!(out, "{i2}hasState: {}", facts.has_state);
    let _ = writeln!(out, "{i2}hasProps: {}", facts.has_props);
    let _ = writeln!(out, "{i2}passesProps: {}", facts.passes_props);
    let _ = writeln!(out, "{i2}usesMemo: {}", facts.uses_memo);
    let _ = writeln!(
        out,
        "{i2}usesContextProvider: {}",
        facts.uses_context_provider
    );
    let _ = writeln!(out, "{i2}usesSuspense: {}", facts.uses_suspense);
    let _ = writeln!(out, "{i2}hasFetch: {}", !facts.fetches.is_empty());
    let _ = writeln!(out, "{i2}environment: {}", facts.environment);
    if !facts.children.is_empty() {
        let _ = writeln!(out, "{i2}children:");
        for child in &facts.children {
            let _ = writeln!(out, "{i2}  {}#{}", child.file, child.name);
        }
    }
    if !facts.dependencies.is_empty() {
        let _ = writeln!(out, "{i2}dependencies:");
        for dep in &facts.dependencies {
            let _ = writeln!(out, "{i2}  {dep}");
        }
    }
    if let Some(agg) = &facts.inherited_from_children {
        let _ = writeln!(out, "{i2}inheritedFromChildren:");
        let i3 = "  ".repeat(current_depth + 2);
        let _ = writeln!(out, "{i3}hasFetch: {}", agg.has_fetch);
    }
    if current_depth < max_depth {
        // In a full recursive implementation, child ComponentFacts would be resolved
        // and printed here. For now children are listed as refs only.
    }
}

#[cfg(test)]
mod tests;

pub fn print_violations(violations: &[Violation]) {
    print!("{}", render_violations(violations));
}

fn render_violations(violations: &[Violation]) -> String {
    let mut out = String::new();
    for violation in violations {
        if let Some(detail) = &violation.detail {
            let _ = writeln!(
                out,
                "{}#{}: {} {detail}",
                violation.file, violation.component, violation.rule
            );
        } else {
            let _ = writeln!(
                out,
                "{}#{}: {}",
                violation.file, violation.component, violation.rule
            );
        }
    }
    out
}

pub fn print_results_md(results: &[ComponentFacts]) {
    print!("{}", render_results_md(results));
}

fn render_results_md(results: &[ComponentFacts]) -> String {
    let mut out = String::from("# React traits\n");
    for result in results {
        let _ = writeln!(
            out,
            "- `{}` `{}` ({})",
            result.file, result.name, result.environment
        );
    }
    out
}

pub fn print_violations_md(violations: &[Violation]) {
    print!("{}", render_violations_md(violations));
}

fn render_violations_md(violations: &[Violation]) -> String {
    let mut out = String::from("# React trait violations\n");
    for violation in violations {
        let _ = writeln!(
            out,
            "- `{}` `{}`: {}",
            violation.file, violation.component, violation.rule
        );
    }
    out
}

fn target_label(report: &UsagesReport) -> String {
    match &report.target.symbol {
        Some(symbol) => format!("{}#{}", report.target.file, symbol),
        None => report.target.file.clone(),
    }
}

fn callsite_props(callsite: &crate::react_traits::report::types::Callsite) -> String {
    let mut parts = callsite.props.clone();
    if callsite.has_spread {
        parts.push("...".to_string());
    }
    parts.join(", ")
}

pub fn print_usages(report: &UsagesReport) {
    print!("{}", render_usages(report));
}

fn render_usages(report: &UsagesReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}", target_label(report));
    let _ = writeln!(out, "  callsites:");
    for callsite in &report.callsites {
        let _ = writeln!(
            out,
            "    {}:{} <{}> [{}]",
            callsite.file,
            callsite.line,
            callsite.component,
            callsite_props(callsite)
        );
    }
    render_usages_section(&mut out, "stories", report.stories.as_deref());
    render_usages_section(&mut out, "tests", report.tests.as_deref());
    render_usages_section(&mut out, "propTypes", report.prop_types.as_deref());
    out
}

fn render_usages_section(out: &mut String, label: &str, items: Option<&[String]>) {
    if let Some(items) = items {
        let _ = writeln!(out, "  {label}:");
        for item in items {
            let _ = writeln!(out, "    {item}");
        }
    }
}

pub fn print_usages_md(report: &UsagesReport) {
    print!("{}", render_usages_md(report));
}

fn render_usages_md(report: &UsagesReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# React usages: `{}`", target_label(report));
    let _ = writeln!(out, "## Callsites");
    for callsite in &report.callsites {
        let _ = writeln!(
            out,
            "- `{}:{}` `<{}>` ({})",
            callsite.file,
            callsite.line,
            callsite.component,
            callsite_props(callsite)
        );
    }
    render_usages_md_section(&mut out, "Stories", report.stories.as_deref());
    render_usages_md_section(&mut out, "Tests", report.tests.as_deref());
    render_usages_md_section(&mut out, "Prop types", report.prop_types.as_deref());
    out
}

fn render_usages_md_section(out: &mut String, label: &str, items: Option<&[String]>) {
    if let Some(items) = items {
        let _ = writeln!(out, "## {label}");
        for item in items {
            let _ = writeln!(out, "- `{item}`");
        }
    }
}
