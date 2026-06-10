#[cfg(test)]
mod tests_types {
    use crate::codebase::dependencies::graph::NodeId;
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
    }
}
