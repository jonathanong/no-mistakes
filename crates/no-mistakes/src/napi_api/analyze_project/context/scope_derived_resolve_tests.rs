use super::concrete_file_closure_args;
use crate::codebase::dependencies::TraverseArgs;

#[test]
fn concrete_file_closure_args_undoes_folder_and_target_module_projections() {
    let args = TraverseArgs {
        filters: vec!["src/**".to_string(), "lib/".to_string()],
        target_modules: vec!["@react/*".to_string()],
        ..TraverseArgs::default()
    };
    let concrete = concrete_file_closure_args(&args);
    assert_eq!(
        concrete.filters,
        vec!["src/**".to_string(), "lib/**".to_string()]
    );
    assert!(concrete.target_modules.is_empty());
    assert_eq!(args.target_modules, vec!["@react/*".to_string()]);
}
