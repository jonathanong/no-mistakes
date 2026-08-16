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

        let symbol_node = NodeId::symbol(file_path.clone(), "MyClass".to_string());
        assert_eq!(symbol_node.as_file(), Some(file_path.as_path()));
        assert_eq!(symbol_node.as_path(), Some(file_path.as_path()));

        let module_node = NodeId::Module("react".to_string());
        assert_eq!(module_node.as_file(), None);
        assert_eq!(module_node.as_path(), None);

        let queue_node = NodeId::queue_job(file_path.clone(), "jobName".to_string());
        assert_eq!(queue_node.as_file(), None);
        assert_eq!(queue_node.as_path(), Some(file_path.as_path()));

        let workflow_job = NodeId::workflow_job(file_path.clone(), "build".to_string());
        assert_eq!(workflow_job.as_file(), None);
        assert_eq!(workflow_job.as_path(), Some(file_path.as_path()));

        let workflow_step = NodeId::workflow_step(file_path, "build".to_string(), 0);
        assert_eq!(workflow_step.as_file(), None);
        assert_eq!(workflow_step.as_path(), Some(Path::new("src/index.ts")));
    }

    #[test]
    fn interned_file_and_symbol_nodes_compare_hash_and_display_equal() {
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
        let symbol_b = NodeId::symbol(right, "Widget".to_string());
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
                assert_eq!(symbol, "Widget");
            }
            other => panic!("expected Symbol, got {other:?}"),
        }
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
