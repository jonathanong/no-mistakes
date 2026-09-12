use super::*;

#[test]
fn resource_callable_ids_match_import_extraction_for_variable_values_and_class_fields() {
    let source = r#"
        import * as fs from 'node:fs';
        export const load = () => fs.readFile('variable.json');
        export class Service { field = fs.readFile('class-field.json'); }
    "#;
    let allocator = oxc_allocator::Allocator::default();
    let parsed = crate::ast::parse(
        std::path::Path::new("resource.ts"),
        &allocator,
        source,
        oxc_span::SourceType::ts(),
    );
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let imports =
        crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source(
            &parsed.program,
            source,
        );
    let resources = extract(&parsed.program, source);

    for (path, scope) in [("variable.json", "load"), ("class-field.json", "Service")] {
        let resource_id = resources
            .calls
            .iter()
            .find(|call| call.path.value == path)
            .and_then(|call| call.function_scope_id)
            .expect("resource call must carry a callable identity");
        assert!(imports
            .callable_scope_ids
            .iter()
            .any(|(id, candidate)| *id == resource_id && candidate == scope));
    }
}

#[test]
fn records_static_url_forms_and_scoped_dynamic_diagnostics() {
    let facts = facts(include_str!(
        "../../../../../../fixtures/test-plan/resource-impact/extractor-coverage.ts"
    ));
    let kinds = facts.calls.iter().map(|call| call.kind).collect::<Vec<_>>();
    for kind in [
        ResourceCallKind::ReadFile,
        ResourceCallKind::ReadFileSync,
        ResourceCallKind::ReadDirectory,
        ResourceCallKind::ReadDirectorySync,
        ResourceCallKind::Glob,
        ResourceCallKind::GlobSync,
    ] {
        assert!(kinds.contains(&kind));
    }
    let diagnostic_kinds = facts
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.kind)
        .collect::<Vec<_>>();
    for kind in [
        ResourceDiagnosticKind::DynamicPath,
        ResourceDiagnosticKind::DynamicPattern,
        ResourceDiagnosticKind::DynamicCwd,
    ] {
        assert!(diagnostic_kinds.contains(&kind));
    }
    for path in [
        "./url-resource.json",
        "./file-url-resource.json",
        "./namespace-url-resource.json",
        "./inline-url-resource.json",
    ] {
        let call = facts
            .calls
            .iter()
            .find(|call| call.path.value == path)
            .unwrap();
        assert_eq!(call.path.base, ResourcePathBase::SourceModule, "{path}");
        assert_eq!(call.function_scope.as_deref(), Some("resourceScope"));
    }
    assert!(facts
        .calls
        .iter()
        .any(|call| call.path.value == "after-var-binding.json"
            && call.path.base == ResourcePathBase::AnalysisRoot
            && call.function_scope.as_deref() == Some("resourceScope")));
    for path in [
        "direct-import.json",
        "direct-sync-import.json",
        "direct-directory-import.json",
        "promises-import.json",
        "promises-namespace-directory.json",
        "inline-require.json",
        "inline-promises-require.json",
        "nested-fs-promises.json",
        "anonymous-arrow.json",
        "switch-resource.json",
        "named-default.json",
        "namespace/**/*.txt",
        "fast-glob/**/*.txt",
        "inline-default-glob/**/*.txt",
        "inline-glob-sync/**/*.txt",
    ] {
        assert!(
            facts.calls.iter().any(|call| call.path.value == path),
            "{path}"
        );
    }
    assert!(facts.calls.iter().any(|call| {
        call.path.value == "templates/**/*.txt"
            && call
                .cwd
                .as_ref()
                .is_some_and(|cwd| cwd.value == "static-cwd")
    }));
    assert!(facts.calls.iter().any(|call| {
        call.path.value == "templates/**/*.txt"
            && call
                .cwd
                .as_ref()
                .is_some_and(|cwd| cwd.base == ResourcePathBase::SourceModule)
    }));
    assert!(facts
        .calls
        .iter()
        .all(|call| call.path.value != "must-not-be-recorded.json"));
    assert_eq!(
        facts
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.kind, diagnostic.function_scope.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            (ResourceDiagnosticKind::DynamicPath, Some("resourceScope")),
            (
                ResourceDiagnosticKind::DynamicPattern,
                Some("resourceScope")
            ),
            (ResourceDiagnosticKind::DynamicCwd, Some("resourceScope")),
        ]
    );
}

#[test]
fn nested_assignment_targets_are_walked() {
    let facts = facts(include_str!(
        "../../../../../../fixtures/test-plan/resource-impact/extractor-nested-assign.ts"
    ));
    assert!(
        facts.calls.is_empty(),
        "nested assignment targets must invalidate rebound fs bindings: {facts:#?}"
    );
}

#[test]
fn default_export_and_argumentless_resource_calls_are_walked() {
    let prefix = "import * as fs from 'node:fs';\nimport { glob } from 'glob';\n";
    for source in [
        "export default function () { fs.readFile('anon-default.json'); }",
        "export default function Named() { fs.readFile('named-default.json'); }",
        "export default () => { fs.readFile('arrow-default.json'); }",
        "export default class C { read() { fs.readFile('class-default.json'); } }",
        "export default (function () { fs.readFile('paren-fn.json'); });",
        "export default (() => { fs.readFile('paren-arrow.json'); });",
        "export default (((function () { fs.readFile('nested-paren-fn.json'); })));",
        "export default (((() => { fs.readFile('nested-paren-arrow.json'); })));",
        "export default (1);\n(fs.readFile)('paren-callee.json');",
        "fs.readFile();\nglob();",
    ] {
        let facts = facts(&format!("{prefix}{source}"));
        assert!(
            !facts.calls.is_empty() || !facts.diagnostics.is_empty(),
            "{source}: {facts:#?}"
        );
    }
}

#[test]
fn require_glob_and_destructure_shapes_are_extracted() {
    let facts = facts(
        r#"
        import { glob as tinyGlob } from 'tinyglobby';
        import { fileURLToPath } from 'node:url';
        import { parse } from 'node:url';
        import * as url from 'node:url';
        import * as fsp from 'node:fs/promises';
        import * as g from 'glob';
        import { globSync } from 'glob';
        const fs = require('fs');
        const { readFile: rf = fs.readFile } = require('fs');
        const [fsArr] = require('fs');
        const dynamicRequire = require(mod);
        const promises = require('fs').promises;
        tinyGlob('tiny/**/*.txt');
        fs.readFile(fileURLToPath(new URL('./via-url.json', import.meta.url)));
        fs.readFile(('paren.json'));
        g.glob('templates/**/*.txt', { cwd: import.meta.dirname });
        g.glob(`literal-cwd/**/*.txt`, { cwd: `static-cwd` });
        parse('https://example.test');
        promises.readdir('dir');
        fsp.readdir('fsp-dir');
        g.sync('g-sync/**/*.txt');
        globSync('glob-sync/**/*.txt');
        url.fileURLToPath(new URL('./ns-url.json', import.meta.url));
        (fileURLToPath)(new URL('./paren-url.json', import.meta.url));
        const { [computed]: skipped } = require('fs');
        const { promises: { readFile: nestedRead } } = require('fs');
        const { promises: { [dyn]: x } } = require('fs');
        const { readFile: { inner } } = require('fs');
        const other = require('other').promises;
        const { foo } = require('fs');
        g.glob('spread/**/*.txt', { ...opts });
        g.glob('computed-cwd/**/*.txt', { [k]: 1 });
        g.glob('dynamic-cwd/**/*.txt', { cwd: dynamic });
        g.glob('paren-cwd/**/*.txt', { cwd: ('static-cwd') });
        g.glob('numeric-cwd/**/*.txt', 1);
        "#,
    );
    assert!(
        facts
            .calls
            .iter()
            .any(|call| call.path.value.contains("tiny"))
            || !facts.diagnostics.is_empty(),
        "{facts:#?}"
    );
}

#[test]
fn module_level_object_and_unnamed_default_class_are_walked() {
    let facts = facts(
        r#"
        import * as fs from 'node:fs';
        const bundle = {
            read() {
                fs.readFile('object-method.json');
            },
        };
        const Ctor = class {
            read() {
                fs.readFile('class-expr.json');
            }
        };
        export default class {
            read() {
                fs.readFile('anon-class.json');
            }
        }
        "#,
    );
    assert!(
        facts.calls.iter().any(|call| {
            matches!(
                call.path.value.as_str(),
                "object-method.json" | "class-expr.json" | "anon-class.json"
            )
        }),
        "{facts:#?}"
    );
}

#[test]
fn glob_cwd_rejects_spreads_and_records_parenthesized_url_and_dirname() {
    let facts = facts(
        r#"
        import * as fs from 'node:fs';
        import { glob } from 'glob';
        import { fileURLToPath } from 'node:url';
        const URL = String;
        fs.readFile(new URL('./shadowed.json', import.meta.url));
        glob('spread-cwd/**/*.txt', { ...opts, cwd: 'x' });
        glob('no-cwd/**/*.txt');
        glob('paren-url/**/*.txt', { cwd: ('static-cwd') });
        glob('meta-cwd/**/*.txt', { cwd: import.meta.dirname });
        glob('tpl-cwd/**/*.txt', { cwd: `tpl` });
        import { URL as UrlCtor } from 'node:url';
        fs.readFile(fileURLToPath(new UrlCtor('./bound-url.json', import.meta.url)));
        fs.readFile(require('url').fileURLToPath(new UrlCtor('./req-url.json', import.meta.url)));
        fs.readFile(new UrlCtor('./imported-url.json', import.meta.url));
        "#,
    );
    assert!(facts
        .diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.kind == ResourceDiagnosticKind::DynamicCwd }));
    assert!(facts
        .calls
        .iter()
        .any(|call| call.path.value.contains("no-cwd") && call.cwd.is_none()));
    assert!(facts.calls.iter().any(|call| {
        call.path.value.contains("paren-url")
            && call
                .cwd
                .as_ref()
                .is_some_and(|cwd| cwd.value == "static-cwd")
    }));
    assert!(facts.calls.iter().any(|call| {
        call.path.value.contains("meta-cwd")
            && call
                .cwd
                .as_ref()
                .is_some_and(|cwd| cwd.base == ResourcePathBase::SourceModule)
    }));
    assert!(facts.calls.iter().any(|call| {
        call.path.value.contains("tpl-cwd")
            && call.cwd.as_ref().is_some_and(|cwd| cwd.value == "tpl")
    }));
    assert!(facts
        .calls
        .iter()
        .any(|call| call.path.value.contains("bound-url.json")));
    assert!(facts
        .calls
        .iter()
        .any(|call| call.path.value.contains("req-url.json")));
    assert!(facts
        .calls
        .iter()
        .all(|call| call.path.value != "./shadowed.json"));
}

#[test]
fn remaining_url_and_glob_argument_shapes() {
    let facts = facts(
        r#"
        import * as fs from 'node:fs';
        import { glob } from 'glob';
        import { fileURLToPath } from 'node:url';
        import { URL } from 'node:url';
        fs.readFile(new URL('./direct-url.json', import.meta.url));
        fs.readFile(fileURLToPath(new URL('./call-url.json', import.meta.url)));
        fs.readFile((fileURLToPath)(new URL('./paren-callee-url.json', import.meta.url)));
        glob('no-arg-cwd/**/*.txt');
        glob('numeric-cwd/**/*.txt', 1);
        glob('spread-only/**/*.txt', { ...opts });
        glob('computed-key/**/*.txt', { [k]: 'x' });
        glob('paren-path/**/*.txt', { cwd: ('static-cwd') });
        glob(`quasi-cwd/**/*.txt`, { cwd: `tpl-cwd` });
        fs.readFile(`template.json`);
        "#,
    );
    assert!(
        facts
            .calls
            .iter()
            .any(|call| call.path.value.contains("direct-url")
                || call.path.value.contains("call-url")
                || call.path.value.contains("no-arg-cwd")
                || !facts.diagnostics.is_empty()),
        "{facts:#?}"
    );
}
