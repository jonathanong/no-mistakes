#![expect(
    clippy::disallowed_methods,
    reason = "Throwaway spike benchmark binary times tokenization passes directly."
)]

use cpd_core::summary::{file_complexity, span_complexity};
use cpd_tokenizer::tokenizer::{tokenize, tokenize_to_detection, Mode, TokenizeOptions};
use no_mistakes::spike_4800::lcov_probes::{probe_edge_cases, summarize_tracefiles};
use no_mistakes::spike_4800::oxc_reuse::{collect_functions, parse_and_tokenize_oxc_0159};
use no_mistakes::spike_4800::{detect_clones_core_only, format_for_path, measure_file_complexity};
use oxc_allocator::Allocator;
use oxc_span::SourceType;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

const SAMPLE_20: &[&str] = &[
    "test-cases/eslint-plugin/test-no-delayed-rejects/fixture/valid.ts",
    "fixtures/postgres-facts/embedded/programs/exec.ts",
    "fixtures/postgres-facts/embedded/variants-switch-inert-generators.ts",
    "fixtures/test-plan/vitest-setup-dependencies/vitest.config.ts",
    "test-cases/rules/postgres-lock-written-rows-only/fixture/conditional-close-coverage/src/query.ts",
    "test-cases/rules/server-route-client-boundary/fixture/client-aliases/backend/api/client.ts",
    "fixtures/postgres-facts/embedded/variants-namespace-member-guards.ts",
    "test-cases/integration-tests/coverage/fixture/playwright.edge.ts",
    "test-cases/rules/postgres-grouping-not-aggregation/fixture/src/shape-controls.ts",
    "test-cases/performance/call-site-assembly/unit-268.tsx",
    "packages/no-mistakes/index.js",
    "packages/no-mistakes/planning-impact-artifacts.js",
    "packages/no-mistakes/planning-impact-artifacts-files.js",
    "packages/no-mistakes/workflow-topology-index.js",
    "packages/no-mistakes/scripts/native-package.js",
    "packages/no-mistakes/scripts/wait-npm-tarball.js",
    "packages/eslint-plugin-no-mistakes/src/rules/async-source-ts-expressions.js",
    "packages/eslint-plugin-no-mistakes/src/rules/async-call-disposition.js",
    "packages/eslint-plugin-no-mistakes/src/rules/async-consumer-proof.js",
    "packages/no-mistakes/test-timeout-policy/index.mjs",
];

fn collect_ts_js_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if p.is_dir() && !matches!(name, "node_modules" | "target" | ".git") {
            collect_ts_js_files(&p, out);
        } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            if matches!(ext, "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs")
                && !name.ends_with(".d.ts")
                && !name.ends_with(".d.mts")
            {
                out.push(p);
            }
        }
    }
}

fn load_scc_map(path: &Path) -> HashMap<String, u64> {
    let mut map = HashMap::new();
    let Ok(text) = std::fs::read_to_string(path) else {
        return map;
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) else {
        return map;
    };
    for lang in val.as_array().into_iter().flatten() {
        for f in lang
            .get("Files")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            if let (Some(loc), Some(cx)) = (
                f.get("Location").and_then(|v| v.as_str()),
                f.get("Complexity").and_then(|v| v.as_u64()),
            ) {
                map.insert(loc.to_string(), cx);
            }
        }
    }
    map
}

fn run_tokenization_benchmark(root: &Path) {
    let mut paths = Vec::new();
    collect_ts_js_files(&root.join("packages"), &mut paths);
    collect_ts_js_files(&root.join("fixtures"), &mut paths);
    paths.sort();
    let files: Vec<(PathBuf, &'static str, String)> = paths
        .into_iter()
        .filter_map(|p| {
            let src = std::fs::read_to_string(&p).ok()?;
            Some((p.clone(), format_for_path(&p), src))
        })
        .collect();
    let bytes: usize = files.iter().map(|(_, _, s)| s.len()).sum();
    println!(
        "=== Q2 BENCH ({} files, {:.2} MiB) ===",
        files.len(),
        bytes as f64 / 1_048_576.0
    );
    let t0 = Instant::now();
    let mut fns = 0usize;
    for (p, _, s) in &files {
        let a = Allocator::new();
        let st = SourceType::from_path(p).unwrap_or_else(|_| SourceType::tsx());
        fns += collect_functions(&no_mistakes_ts_facts::ast::parse(p, &a, s, st).program, s).len();
    }
    let m0 = t0.elapsed().as_secs_f64() * 1000.0;
    let t1 = Instant::now();
    let n1: usize = files
        .iter()
        .map(|(p, _, s)| parse_and_tokenize_oxc_0159(p, s).0.len())
        .sum();
    let m1 = t1.elapsed().as_secs_f64() * 1000.0;
    let opts = TokenizeOptions::new(Mode::Mild);
    let t2 = Instant::now();
    let n2: usize = files
        .iter()
        .map(|(_, f, s)| tokenize_to_detection(f, s, &opts).len())
        .sum();
    let m2 = t2.elapsed().as_secs_f64() * 1000.0;
    let t3 = Instant::now();
    let n3: usize = files
        .iter()
        .map(|(_, f, s)| tokenize(f, s, Mode::Mild).len())
        .sum();
    let m3 = t3.elapsed().as_secs_f64() * 1000.0;
    println!("  oxc159 parse+ast: {m0:.2} ms (fns={fns})");
    println!(
        "  oxc159 reused:    {m1:.2} ms (toks={n1}, delta=+{:.2} ms)",
        m1 - m0
    );
    println!(
        "  cpd det_tokens:   {m2:.2} ms (toks={n2}, total={:.2} ms)",
        m0 + m2
    );
    println!("  cpd Vec<Token>:   {m3:.2} ms (toks={n3})");
}

fn run_complexity_comparison(root: &Path) {
    let scc_map = load_scc_map(Path::new("/tmp/spike-4800-scc.json"));
    println!("=== Q3 COMPLEXITY PARITY (20 files) ===");
    for rel in SAMPLE_20 {
        let full = root.join(rel);
        let src = std::fs::read_to_string(&full).expect("read");
        let r = measure_file_complexity(&full, &src);
        let scc = scc_map.get(*rel).copied().unwrap_or(0);
        println!(
            "{rel} | {scc} | {} | {} | {} | {} | {} | {}",
            r.cpd_file_complexity,
            r.span_complexity_sum,
            r.span_complexity_max,
            r.ast_complexity_sum,
            r.ast_complexity_max,
            r.function_count
        );
    }
    let probes: &[(&str, &str)] = &[
        ("plain", "export function f(x: number): number { return x + 1; }"),
        ("kw_idents", "export function f(or: number, and: number, when: number) { return or + and + when; }"),
        ("obj_keys", "export function f() { return { when: 1, cond: 2, or: 3, and: 4, until: 5, unless: 6 }; }"),
        ("ts_arrows", "export type A = (x: number) => (y: string) => void;\nexport function f(cb: (a: number) => void) { return cb(1); }"),
        ("ts_cond", "export type IsStr<T> = T extends string ? true : false;\nexport function f(a?: string, b?: { c?: number }) { return b?.c; }"),
        ("nested_cb", "export function outer(xs: number[]) { if (xs.length > 0) { return xs.filter(x => x > 1 && x < 10); } return []; }"),
    ];
    for (name, code) in probes {
        let toks = tokenize("typescript", code, Mode::Mild);
        let fc = file_complexity(&toks, "typescript");
        let alloc = Allocator::new();
        let parsed =
            no_mistakes_ts_facts::ast::parse(Path::new("p.ts"), &alloc, code, SourceType::ts());
        let spans: Vec<_> = collect_functions(&parsed.program, code)
            .into_iter()
            .map(|f| {
                (
                    f.name,
                    span_complexity(&toks, "typescript", f.start, f.end),
                    f.ast_complexity,
                )
            })
            .collect();
        println!("  probe {name}: file_cx={fc} funcs={spans:?}");
    }
}

fn main() {
    let root = std::env::current_dir().expect("cwd");
    let pkg: Vec<_> = SAMPLE_20
        .iter()
        .filter(|p| p.starts_with("packages/"))
        .map(|rel| {
            let full = root.join(rel);
            (
                rel.to_string(),
                format_for_path(&full).to_string(),
                std::fs::read_to_string(&full).expect("read"),
            )
        })
        .collect();
    println!("core_only_clones={}", detect_clones_core_only(&pkg, 30, 3));
    run_tokenization_benchmark(&root);
    run_complexity_comparison(&root);
    let vitest: Vec<PathBuf> = [
        "/tmp/spike-4800-vitest-cov/lcov.info",
        "/tmp/spike-4800-vitest-cov-2/lcov.info",
        "/tmp/spike-4800-vitest-cov-3/lcov.info",
    ]
    .into_iter()
    .map(PathBuf::from)
    .filter(|p| p.exists())
    .collect();
    summarize_tracefiles("Vitest v8", &vitest);
    let llvm: Vec<PathBuf> = [
        "/tmp/spike-4800-llvm-cov-1.lcov",
        "/tmp/spike-4800-llvm-cov-2.lcov",
    ]
    .into_iter()
    .map(PathBuf::from)
    .filter(|p| p.exists())
    .collect();
    summarize_tracefiles("llvm-cov", &llvm);
    for (name, outcome) in probe_edge_cases() {
        println!("  edge {name}: {outcome}");
    }
}
