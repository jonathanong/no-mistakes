use super::*;

#[test]
fn retained_invalid_bytes_honor_line_and_next_line_directives_for_every_rule() {
    let root = saved();
    for &(rule, _) in CASES {
        for (name, line) in [("line-next", 2), ("line-same", 1)] {
            let mut config = config(&root, rule);
            config.rules[0].include =
                vec![config.rules[0].include[0].replace("bad.", &format!("{name}."))];
            let files = files(&root, &config);
            let sources = super::super::super::source_store_for_files(&files);
            let output = evaluate(&root, &config, &files, &sources, false);
            assert!(
                output.findings.is_empty() && output.advisories.is_empty(),
                "{rule}/{name}: {:?}",
                output.findings
            );
            let audited = evaluate(&root, &config, &files, &sources, true);
            assert_eq!(audited.findings.len(), 1, "{rule}/{name}");
            assert_eq!(audited.findings[0].line, line, "{rule}/{name}");
            assert_eq!(sources.physical_read_count(), 1, "{rule}/{name}");
        }
    }
}
