use super::super::test_flake_helpers::{
    ast::{array_objects, find_property, project_name},
    config_ast::extract_config_objects,
    is_script,
};
use super::options::CompiledOptions;
use super::scan_ts_calls::{check_retry_expr, check_vitest_retry, scan_calls};
use crate::codebase::rules::{self, RuleFinding};
use crate::codebase::ts_source::{unwrap_ts_wrappers, SourceStore};
use oxc_ast::ast::{Expression, ObjectExpression};
use std::path::Path;

pub(super) fn scan_file(
    path: &Path,
    rel: &str,
    opts: &CompiledOptions,
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    if !is_script(rel) {
        return Vec::new();
    }
    let Some(source) = rules::read_language_source(sources, path) else {
        return Vec::new();
    };
    scan_source(rel, &source, opts)
}

pub(super) fn scan_source(file: &str, source: &str, opts: &CompiledOptions) -> Vec<RuleFinding> {
    let is_pw = opts.is_playwright_config(file);
    let is_vitest = opts.is_vitest_config(file);
    crate::ast::with_program(Path::new(file), source, |program, source| {
        let mut findings = Vec::new();
        if is_pw || is_vitest {
            for obj in extract_config_objects(program) {
                if is_pw {
                    scan_playwright_config(file, source, obj, opts, &mut findings);
                }
                if is_vitest {
                    scan_vitest_config(file, source, obj, opts, &mut findings);
                }
            }
        }
        if !opts.allow.is_allowed(file, None) {
            scan_calls(file, source, program, opts.max_retries, &mut findings);
        }
        findings
    })
    .unwrap_or_default()
}

fn scan_playwright_config(
    file: &str,
    source: &str,
    obj: &ObjectExpression<'_>,
    opts: &CompiledOptions,
    findings: &mut Vec<RuleFinding>,
) {
    if !opts.allow.is_allowed(file, None) {
        if let Some(prop) = find_property(obj, "retries") {
            check_retry_expr(
                file,
                source,
                &prop.value,
                opts.max_retries,
                "Playwright config `retries`",
                "retries",
                findings,
            );
        }
    }
    let Some(projects) = find_property(obj, "projects") else {
        return;
    };
    for proj in array_objects(&projects.value) {
        let name = project_name(proj);
        if opts.allow.is_allowed(file, name) {
            continue;
        }
        let Some(prop) = find_property(proj, "retries") else {
            continue;
        };
        let label = match name {
            Some(name) => format!("Playwright project `{name}` `retries`"),
            None => "Playwright project `retries`".to_string(),
        };
        check_retry_expr(
            file,
            source,
            &prop.value,
            opts.max_retries,
            &label,
            "retries",
            findings,
        );
    }
}

fn scan_vitest_config(
    file: &str,
    source: &str,
    obj: &ObjectExpression<'_>,
    opts: &CompiledOptions,
    findings: &mut Vec<RuleFinding>,
) {
    let Some(Expression::ObjectExpression(test_obj)) =
        find_property(obj, "test").map(|prop| unwrap_ts_wrappers(&prop.value))
    else {
        return;
    };
    if !opts.allow.is_allowed(file, None) {
        if let Some(prop) = find_property(test_obj, "retry") {
            check_vitest_retry(
                file,
                source,
                &prop.value,
                opts.max_retries,
                "Vitest config `test.retry`",
                findings,
            );
        }
    }
    let Some(projects) = find_property(test_obj, "projects") else {
        return;
    };
    for proj in array_objects(&projects.value) {
        let name = project_name(proj);
        if opts.allow.is_allowed(file, name) {
            continue;
        }
        let inner =
            find_property(proj, "test").and_then(|prop| match unwrap_ts_wrappers(&prop.value) {
                Expression::ObjectExpression(inner) => Some(inner.as_ref()),
                _ => None,
            });
        let Some(prop) = inner
            .and_then(|obj| find_property(obj, "retry"))
            .or_else(|| find_property(proj, "retry"))
        else {
            continue;
        };
        let label = match name {
            Some(name) => format!("Vitest project `{name}` `retry`"),
            None => "Vitest project `retry`".to_string(),
        };
        check_vitest_retry(
            file,
            source,
            &prop.value,
            opts.max_retries,
            &label,
            findings,
        );
    }
}
