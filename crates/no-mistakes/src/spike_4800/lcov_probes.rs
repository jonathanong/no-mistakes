use lcov::{Reader, Report};
use std::path::PathBuf;

pub fn probe_edge_cases() -> Vec<(&'static str, String)> {
    let cases: &[(&str, &str)] = &[
        (
            "trailing_whitespace",
            "SF:a.ts  \nDA:1,1  \nend_of_record  \n",
        ),
        ("blank_line", "\nSF:a.ts\nDA:1,1\nend_of_record\n"),
        (
            "unknown_record",
            "SF:a.ts\nZZZ:ignored\nDA:1,1\nend_of_record\n",
        ),
        ("missing_eor_at_eof", "SF:a.ts\nDA:1,1\n"),
        (
            "duplicate_sf_same_file",
            "SF:a.ts\nDA:1,1\nend_of_record\nSF:a.ts\nDA:1,2\nend_of_record\n",
        ),
        (
            "duplicate_da_same_section",
            "SF:a.ts\nDA:1,1\nDA:1,2\nend_of_record\n",
        ),
        (
            "fn_3_field_lcov2",
            "SF:a.ts\nFN:1,5,myFn\nFNDA:2,myFn\nend_of_record\n",
        ),
        (
            "fnl_fna_lcov2",
            "SF:a.ts\nFNL:0,1,5\nFNA:0,2,myFn\nend_of_record\n",
        ),
        (
            "brda_non_numeric_block",
            "SF:a.ts\nBRDA:10,blockA,0,3\nend_of_record\n",
        ),
        (
            "brda_5_fields",
            "SF:a.ts\nBRDA:10,0,0,3,extra\nend_of_record\n",
        ),
        (
            "brda_dash_untaken",
            "SF:a.ts\nBRDA:10,0,0,-\nend_of_record\n",
        ),
        ("empty_sf_section", "SF:a.ts\nend_of_record\n"),
        (
            "vitest_empty_report_placeholder",
            "SF:a.ts\nFN:1,(empty-report)\nFNDA:0,(empty-report)\nend_of_record\n",
        ),
        (
            "fn_after_fnda_clobber",
            "SF:a.ts\nFNDA:5,foo\nFN:1,foo\nend_of_record\n",
        ),
    ];

    let mut results = Vec::new();
    for (name, input) in cases {
        let outcome = match Report::from_reader(Reader::new(input.as_bytes())) {
            Err(err) => format!("ERR: {err}"),
            Ok(report) => format_report_brief(&report),
        };
        results.push((*name, outcome));
    }
    let parse_ok = |s: &[u8]| Report::from_reader(Reader::new(s)).unwrap();
    let r1 = parse_ok(b"TN:unit\nSF:a.ts\nFN:1,foo\nFNDA:2,foo\nDA:1,2\nend_of_record\n");
    let r2 = parse_ok(b"TN:e2e\nSF:a.ts\nFN:1,foo\nFNDA:3,foo\nDA:1,3\nend_of_record\n");
    let mut m_tn = r1.clone();
    let _ = m_tn.merge(r2);
    results.push((
        "merge_different_tn",
        format!(
            "sections={} (did_not_merge={})",
            m_tn.sections.len(),
            m_tn.sections.len() == 2
        ),
    ));
    let r3 = parse_ok(b"TN:unit\nSF:a.ts\nFN:9,foo\nFNDA:4,foo\nDA:1,4\nend_of_record\n");
    let mut m_fn = r1.clone();
    let strict = m_fn.merge(r3.clone());
    let mut m_lossy = r1;
    m_lossy.merge_lossy(r3);
    let lossy_line = m_lossy
        .sections
        .values()
        .next()
        .and_then(|s| s.functions.values().next())
        .and_then(|v| v.start_line);
    results.push((
        "merge_fn_line_conflict",
        format!("strict={strict:?} lossy_start_line={lossy_line:?}"),
    ));
    results
}

fn format_report_brief(report: &Report) -> String {
    let mut detail = format!("OK sections={}", report.sections.len());
    for (k, v) in &report.sections {
        let da: Vec<_> = v.lines.iter().map(|(lk, lv)| (lk.line, lv.count)).collect();
        let fns: Vec<_> = v
            .functions
            .iter()
            .map(|(fk, fv)| (fk.name.as_str(), fv.start_line, fv.count))
            .collect();
        let brs: Vec<_> = v
            .branches
            .iter()
            .map(|(bk, bv)| (bk.line, bk.block, bk.branch, bv.taken))
            .collect();
        detail.push_str(&format!(
            " [sf={} da={da:?} fn={fns:?} br={brs:?}]",
            k.source_file.display()
        ));
    }
    detail
}

pub fn summarize_tracefiles(label: &str, paths: &[PathBuf]) {
    println!("=== LCOV {label} ({} files) ===", paths.len());
    let mut merged = Report::new();
    for path in paths {
        match Report::from_file(path) {
            Err(err) => println!("  PARSE ERR {}: {err}", path.display()),
            Ok(rep) => {
                let c = count_report(&rep);
                println!(
                    "  file {} -> sections={} FN={}(hit={}) BRDA={}(hit={},none={}) DA={}(sum={})",
                    path.display(),
                    rep.sections.len(),
                    c.0,
                    c.1,
                    c.3,
                    c.4,
                    c.5,
                    c.7,
                    c.9
                );
                if let Err(err) = merged.merge(rep.clone()) {
                    println!("    strict merge ERR: {err} -> merge_lossy");
                    merged.merge_lossy(rep);
                }
            }
        }
    }
    let c = count_report(&merged);
    println!(
        "  MERGED -> sections={} FN={}(hit={},sum={}) BRDA={}(hit={},sum={}) DA={}(hit={},sum={})",
        merged.sections.len(),
        c.0,
        c.1,
        c.2,
        c.3,
        c.4,
        c.6,
        c.7,
        c.8,
        c.9
    );
}

fn count_report(
    rep: &Report,
) -> (
    usize,
    usize,
    u64,
    usize,
    usize,
    usize,
    u64,
    usize,
    usize,
    u64,
) {
    let mut c = (0, 0, 0u64, 0, 0, 0, 0u64, 0, 0, 0u64);
    for s in rep.sections.values() {
        c.0 += s.functions.len();
        c.1 += s.functions.values().filter(|v| v.count > 0).count();
        c.2 += s.functions.values().map(|v| v.count).sum::<u64>();
        c.3 += s.branches.len();
        c.4 += s
            .branches
            .values()
            .filter(|v| v.taken.unwrap_or(0) > 0)
            .count();
        c.5 += s.branches.values().filter(|v| v.taken.is_none()).count();
        c.6 += s
            .branches
            .values()
            .map(|v| v.taken.unwrap_or(0))
            .sum::<u64>();
        c.7 += s.lines.len();
        c.8 += s.lines.values().filter(|v| v.count > 0).count();
        c.9 += s.lines.values().map(|v| v.count).sum::<u64>();
    }
    c
}
