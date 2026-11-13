use super::*;

#[test]
fn server_commands_execute_saved_routes_and_propagate_invalid_filter_errors() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/server-ast-routes/remix/fixture");
    for (invalid_filter, observed) in [(false, false), (false, true), (true, false), (true, true)] {
        let commands = [
            ServerCommand::Routes {
                files: vec!["/users".into()],
            },
            ServerCommand::Edges {
                roots: vec!["app/routes/users.tsx".into()],
            },
            ServerCommand::Related {
                roots: vec!["app/routes/users.tsx".into()],
                direction: ServerDirection::Deps,
            },
            ServerCommand::Related {
                roots: vec!["/users".into()],
                direction: ServerDirection::Dependents,
            },
            ServerCommand::Related {
                roots: vec!["app/routes/users.tsx".into()],
                direction: ServerDirection::Both,
            },
        ];
        for command in commands {
            let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
            let result = no_mistakes::diagnostics::with_observer(
                observed.then_some(observer.clone()),
                || {
                    run(ServerArgs {
                        common: GraphCommandArgs {
                            root: root.clone(),
                            tsconfig: None,
                            filters: if invalid_filter {
                                vec!["[".into()]
                            } else {
                                Vec::new()
                            },
                            depth: Some(1),
                            format: Format::Human,
                            json: true,
                            timings: false,
                        },
                        command,
                    })
                },
            );
            let timings = observer.snapshot().timings;
            for label in ["analysis.prepare", "analysis.server"] {
                assert_eq!(
                    timings.iter().any(|timing| timing.label == label),
                    observed,
                    "{label}"
                );
            }
            if invalid_filter {
                let error = result.expect_err("invalid filters must stop the selected command");
                assert!(format!("{error:#}").contains("glob"), "{error:#}");
            } else {
                assert_eq!(result.unwrap(), ExitCode::SUCCESS);
            }
        }
    }
}
