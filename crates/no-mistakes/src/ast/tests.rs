use std::path::{Path, PathBuf};

#[test]
fn production_oxc_parses_use_the_observable_chokepoint() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let roots = [
        crate::codebase::ts_resolver::normalize_path(&manifest_dir.join("src")),
        crate::codebase::ts_resolver::normalize_path(
            &manifest_dir.join("../no-mistakes-ts-facts/src"),
        ),
        crate::codebase::ts_resolver::normalize_path(
            &manifest_dir.join("../no-mistakes-ts-analysis/src"),
        ),
    ];
    let mut offenders = Vec::new();
    for src in &roots {
        let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(src);
        let sources = snapshot.source_store_for(src);
        for path in sources.inventory().paths().iter() {
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let relative = path.strip_prefix(src).unwrap();
            if relative == Path::new("ast.rs")
                || relative
                    .components()
                    .any(|component| component.as_os_str() == "tests")
                || relative
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with("_tests.rs") || name == "tests.rs")
            {
                continue;
            }
            let Ok(source) = sources.read_path(path) else {
                continue;
            };
            let lines = source.lines().collect::<Vec<_>>();
            let imports_oxc_parser = source.contains("oxc_parser");
            let has_production_reference = lines.iter().enumerate().any(|(index, line)| {
                let references_parser = line.contains("oxc_parser")
                    || (imports_oxc_parser && line.contains("Parser::new("));
                let test_only_import = index > 0
                    && lines[index - 1].trim() == "#[cfg(test)]"
                    && line.trim_start().starts_with("use ");
                references_parser && !test_only_import
            });
            if has_production_reference {
                offenders.push(
                    path.strip_prefix(&manifest_dir)
                        .unwrap_or(path)
                        .to_path_buf(),
                );
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "production OXC parser entrypoints must call crate::ast::parse: {offenders:?}"
    );
}

pub(crate) fn request_parse_cache_len() -> usize {
    super::request_parse_cache_len()
}
