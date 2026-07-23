#[cfg(test)]
mod tests_types {
    use crate::codebase::dependencies::graph::{EdgeKind, NodeId, VitestSetupField};
    use std::path::PathBuf;

    #[test]
    fn test_nodeid_as_file() {
        let file_path = PathBuf::from("src/index.ts");
        let file_node = NodeId::File(file_path.clone());
        assert_eq!(file_node.as_file(), Some(file_path.as_path()));

        let symbol_node = NodeId::Symbol {
            file: file_path.clone(),
            symbol: "MyClass".to_string(),
        };
        assert_eq!(symbol_node.as_file(), Some(file_path.as_path()));

        let module_node = NodeId::Module("react".to_string());
        assert_eq!(module_node.as_file(), None);

        let queue_node = NodeId::QueueJob {
            queue_file: file_path.clone(),
            job: "jobName".to_string(),
        };
        assert_eq!(queue_node.as_file(), None);

        let workflow_job = NodeId::WorkflowJob {
            workflow_file: file_path.clone(),
            job: "build".to_string(),
        };
        assert_eq!(workflow_job.as_file(), None);

        let workflow_step = NodeId::WorkflowStep {
            workflow_file: file_path,
            job: "build".to_string(),
            step: 0,
        };
        assert_eq!(workflow_step.as_file(), None);
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
