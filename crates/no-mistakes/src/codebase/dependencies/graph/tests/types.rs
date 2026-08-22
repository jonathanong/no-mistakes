#[cfg(test)]
mod tests_types {
    use crate::codebase::dependencies::graph::{EdgeKind, NodeId, VitestSetupField};
    use std::collections::HashSet;
    use std::hash::{Hash, Hasher};
    use std::path::{Path, PathBuf};

    fn hash_of(node: &NodeId) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        node.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn test_nodeid_as_file() {
        let file_path = PathBuf::from("src/index.ts");
        let file_node = NodeId::file(file_path.clone());
        assert_eq!(file_node.as_file(), Some(file_path.as_path()));
        assert_eq!(file_node.as_path(), Some(file_path.as_path()));

        let symbol_node = NodeId::symbol(file_path.clone(), "MyClass");
        assert_eq!(symbol_node.as_file(), Some(file_path.as_path()));
        assert_eq!(symbol_node.as_path(), Some(file_path.as_path()));

        let module_node = NodeId::Module("react".to_string().into());
        assert_eq!(module_node.as_file(), None);
        assert_eq!(module_node.as_path(), None);

        let queue_node = NodeId::queue_job(file_path.clone(), "jobName");
        assert_eq!(queue_node.as_file(), None);
        assert_eq!(queue_node.as_path(), Some(file_path.as_path()));

        let workflow_job = NodeId::workflow_job(file_path.clone(), "build");
        assert_eq!(workflow_job.as_file(), None);
        assert_eq!(workflow_job.as_path(), Some(file_path.as_path()));

        let workflow_step = NodeId::workflow_step(file_path.clone(), "build", 0);
        assert_eq!(workflow_step.as_file(), None);
        assert_eq!(workflow_step.as_path(), Some(Path::new("src/index.ts")));

        let trpc = NodeId::trpc_procedure(file_path, "user.get");
        assert_eq!(trpc.as_file(), None);
        assert_eq!(trpc.as_path(), Some(Path::new("src/index.ts")));
        assert_eq!(
            trpc.display_name(Path::new("src")),
            "index.ts#procedure:user.get"
        );
        let universe: crate::fx::PathSet = [PathBuf::from("src/index.ts")].into_iter().collect();
        assert!(trpc.is_in_file_universe(&universe));
        assert!(!NodeId::trpc_procedure("src/other.ts", "user.get").is_in_file_universe(&universe));
    }

    #[test]
    fn types_node_id_interned_file_and_symbol_nodes_compare_hash_and_display_equal() {
        let left = PathBuf::from("src/widget.ts");
        let right = PathBuf::from("src/widget.ts");
        let file_a = NodeId::file(&left);
        let file_b = NodeId::file(right.clone());
        assert_eq!(file_a, file_b);
        assert_eq!(hash_of(&file_a), hash_of(&file_b));
        assert_eq!(format!("{file_a:?}"), format!("{file_b:?}"));
        assert_eq!(
            file_a.display_name(Path::new("")),
            file_b.display_name(Path::new(""))
        );
        assert_eq!(file_a.display_name(Path::new("src")), "widget.ts");

        let symbol_a = NodeId::symbol(&left, "Widget");
        let symbol_b = NodeId::symbol(right, "Widget");
        assert_eq!(symbol_a, symbol_b);
        assert_eq!(hash_of(&symbol_a), hash_of(&symbol_b));
        assert_eq!(format!("{symbol_a:?}"), format!("{symbol_b:?}"));
        assert_eq!(symbol_a.display_name(Path::new("src")), "widget.ts#Widget");

        let mut set = HashSet::new();
        set.insert(file_a.clone());
        set.insert(symbol_a.clone());
        assert!(set.contains(&file_b));
        assert!(set.contains(&symbol_b));
        match &file_a {
            NodeId::File(path) => assert_eq!(path.as_ref(), Path::new("src/widget.ts")),
            other => panic!("expected File, got {other:?}"),
        }
        match &symbol_a {
            NodeId::Symbol { file, symbol } => {
                assert_eq!(file.as_ref(), Path::new("src/widget.ts"));
                assert_eq!(symbol.as_ref(), "Widget");
            }
            other => panic!("expected Symbol, got {other:?}"),
        }
    }

    #[test]
    fn interned_symbol_and_job_strings_share_arc_on_clone_and_keep_sort_hash() {
        use std::sync::Arc;

        let path = PathBuf::from("src/jobs.ts");
        let symbol = NodeId::symbol(&path, "send");
        let queue = NodeId::queue_job(&path, "send");
        let workflow = NodeId::workflow_job(&path, "send");
        let step = NodeId::workflow_step(&path, "send", 2);

        let symbol_clone = symbol.clone();
        match (&symbol, &symbol_clone) {
            (NodeId::Symbol { symbol: left, .. }, NodeId::Symbol { symbol: right, .. }) => {
                assert!(Arc::ptr_eq(left, right))
            }
            _ => panic!("expected Symbol"),
        }
        let queue_clone = queue.clone();
        match (&queue, &queue_clone) {
            (NodeId::QueueJob { job: left, .. }, NodeId::QueueJob { job: right, .. }) => {
                assert!(Arc::ptr_eq(left, right));
            }
            _ => panic!("expected QueueJob"),
        }
        let workflow_clone = workflow.clone();
        match (&workflow, &workflow_clone) {
            (NodeId::WorkflowJob { job: left, .. }, NodeId::WorkflowJob { job: right, .. }) => {
                assert!(Arc::ptr_eq(left, right))
            }
            _ => panic!("expected WorkflowJob"),
        }
        let step_clone = step.clone();
        match (&step, &step_clone) {
            (NodeId::WorkflowStep { job: left, .. }, NodeId::WorkflowStep { job: right, .. }) => {
                assert!(Arc::ptr_eq(left, right))
            }
            _ => panic!("expected WorkflowStep"),
        }

        assert_eq!(hash_of(&symbol), hash_of(&symbol_clone));
        assert_eq!(hash_of(&queue), hash_of(&queue_clone));
        assert_ne!(hash_of(&symbol), hash_of(&queue));
        assert_eq!(
            symbol.display_name(Path::new("src")),
            queue.display_name(Path::new("src"))
        );
        assert_eq!(workflow.display_name(Path::new("src")), "jobs.ts#job:send");
        assert_eq!(
            step.display_name(Path::new("src")),
            "jobs.ts#job:send/step:2"
        );

        let owned: Arc<str> = Arc::from("reuse");
        let reused = NodeId::symbol(&path, owned.clone());
        match reused {
            NodeId::Symbol { symbol, .. } => assert!(Arc::ptr_eq(&owned, &symbol)),
            other => panic!("expected Symbol, got {other:?}"),
        }
    }

    #[test]
    fn interned_paths_normalize_dot_dotdot_and_trailing_slash() {
        let cases = [
            ("src/./a.ts", "src/a.ts"),
            ("src/foo/../a.ts", "src/a.ts"),
            ("src/a.ts/", "src/a.ts"),
            ("src/./foo/../a.ts", "src/a.ts"),
        ];
        for (raw, expected) in cases {
            let left = NodeId::file(raw);
            let right = NodeId::file(expected);
            assert_eq!(left, right, "eq {raw} vs {expected}");
            assert_eq!(hash_of(&left), hash_of(&right), "hash {raw} vs {expected}");

            let left_path = left.as_path().expect("file node");
            let right_path = right.as_path().expect("file node");
            assert_eq!(
                left_path.as_os_str(),
                right_path.as_os_str(),
                "os-str bytes {raw} vs {expected}"
            );
            assert_eq!(left_path, right_path, "path eq {raw} vs {expected}");
        }
    }

    #[test]
    fn node_id_file_symbol_and_queue_job_on_same_path_are_not_equal() {
        let path = "src/a.ts";
        let file = NodeId::file(path);
        let symbol = NodeId::symbol(path, "job");
        let queue = NodeId::queue_job(path, "job");
        assert_ne!(file, symbol);
        assert_ne!(file, queue);
        assert_ne!(symbol, queue);
        assert_ne!(hash_of(&file), hash_of(&symbol));
        assert_ne!(hash_of(&file), hash_of(&queue));
        assert_ne!(hash_of(&symbol), hash_of(&queue));
        let trpc = NodeId::trpc_procedure(path, "job");
        assert_ne!(file, trpc);
        assert_ne!(queue, trpc);
        assert_ne!(hash_of(&file), hash_of(&trpc));
        assert_ne!(hash_of(&queue), hash_of(&trpc));
    }

    #[test]
    fn module_constructor_matches_interned_variant() {
        let via_ctor = NodeId::module("lodash");
        let via_variant = NodeId::Module("lodash".into());
        let via_string = NodeId::Module("lodash".to_string().into());
        assert_eq!(via_ctor, via_variant);
        assert_eq!(via_ctor, via_string);
        assert_eq!(hash_of(&via_ctor), hash_of(&via_variant));
        assert_eq!(hash_of(&via_ctor), hash_of(&via_string));
    }

    #[test]
    fn edge_kind_sort_key_appends_workflow_and_vitest_kinds_without_reordering_existing_kinds() {
        assert_eq!(EdgeKind::CiInvocation.sort_key(), (14, 0));
        assert_eq!(EdgeKind::TerraformOutputRef.sort_key(), (29, 0));
        assert_eq!(EdgeKind::WorkflowJob.sort_key(), (30, 0));
        assert_eq!(EdgeKind::WorkflowArtifact.sort_key(), (35, 0));
        assert_eq!(
            EdgeKind::VitestSetup(VitestSetupField::SetupFiles).sort_key(),
            (36, 0)
        );
    }
}
