fn run_live_analysis(
    args: &WhyArgs,
    root: &Path,
    test_rel: &str,
) -> Result<BTreeMap<String, Vec<WhyStep>>> {
    // Generate plan live to find all connections and warn/fallback correctly
    let plan_args = PlanArgs {
        framework: None,
        root: root.to_path_buf(),
        config: args.config.clone(),
        tsconfig: args.tsconfig.clone(),
        base: None,
        head: None,
        from_git_diff: None,
        changed_file: args.changed.clone().into_iter().collect(),
        changed_files: None,
        changes: None,
        diff: None,
        diff_stdin: false,
        diff_command: None,
        entrypoints: Vec::new(),
        entrypoint_symbols: Vec::new(),
        include_symbols: false,
        diff_content: None,
        environment: "pre-push".to_string(),
        limit_percent: None,
        limit_files: None,
        global_config_fallback: None,
        direct_test_owner: false,
        format: None,
        json: true,
        include_comment: false,
        include_glob: Vec::new(),
    };

    let plan = generate_plan(&plan_args)?;
    let mut result = BTreeMap::new();

    if let Some(selected) = plan.selected_tests.iter().find(|t| t.test_file == test_rel) {
        for reason in &selected.reasons {
            let mut steps = Vec::new();
            for i in 0..reason.path.len() {
                let node = reason.path[i].clone();
                let via = if i < reason.via.len() {
                    Some(reason.via[i].clone())
                } else {
                    None
                };
                let detail = reason.via_details.get(i).cloned().flatten();
                steps.push(WhyStep { node, via, detail });
            }
            result.insert(reason.changed_file.clone(), steps);
        }
    }

    Ok(result)
}

fn relative_path_str(root: &Path, path: &Path) -> String {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let absolute_normalized = no_mistakes::codebase::ts_resolver::normalize_path(&absolute);
    no_mistakes::codebase::ts_source::relative_slash_path(root, &absolute_normalized)
}
