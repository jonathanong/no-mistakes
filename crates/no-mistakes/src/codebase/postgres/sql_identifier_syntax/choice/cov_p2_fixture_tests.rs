// cspell:ignore rsplit
use super::*;

#[test]
fn cov_p2_canonical_choices_and_carriers_keep_exact_sql_and_opaque_use_spans() {
    let profile = profile(&["read", "write"], &["QueryExecutor", "QueryOptions"]);
    let mut checked = 0;
    for name in [
        "cov-p2-syntax-choices.mts",
        "cov-p2-syntax-methods.mts",
        "cov-p2-syntax-barrier.mts",
        "cov-p2-syntax-global.mjs",
        "cov-p2-syntax-containers.mjs",
    ] {
        let (source, facts) = fixture(name, std::slice::from_ref(&profile));
        let calls = &facts.profiles[0].1;
        for (index, line) in source.lines().enumerate() {
            let rows = calls
                .iter()
                .filter(|call| call.line as usize == index + 1)
                .collect::<Vec<_>>();
            let spans = rows
                .iter()
                .map(|call| {
                    (
                        &source[call.span.0 as usize..call.span.1 as usize],
                        &call.input,
                    )
                })
                .collect::<Vec<_>>();
            if let Some((_, value)) = line.split_once("// SQL:") {
                let (span, sql) = value.rsplit_once("=>").unwrap();
                assert!(
                    spans.iter().any(|(actual, input)| *actual == span
                        && matches!(input, Input::Sql { text, .. } if text == sql)),
                    "{name}: {line}: {spans:?}"
                );
            } else if let Some((_, value)) = line.split_once("// UNKNOWN:") {
                assert!(
                    spans
                        .iter()
                        .any(|(actual, input)| *actual == value && matches!(input, Input::Unknown)),
                    "{name}: {line}: {spans:?}"
                );
            } else if line.ends_with("// QUIET") {
                assert!(rows.is_empty(), "{name}: {line}: {spans:?}");
            } else {
                continue;
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 28);
}
