use super::{has_shebang, scan};

#[test]
fn anonymous_function_hides_status_only_inside_its_scope() {
    let script = "() { local -h status=1; status=2; }\nstatus=3\n";
    assert_eq!(scan(script), vec![("status", 2)]);
}

#[test]
fn hyphenated_and_subshell_functions_preserve_main_zsh_scope() {
    let hyphenated = "foo-bar() {\n  local -h status=1\n  status=2\n}\nstatus=3\n";
    assert_eq!(scan(hyphenated), vec![("status", 5)]);

    let subshell = "f() (\n  local -h status=1\n  status=2\n)\n";
    assert_eq!(scan(subshell), vec![("status", 2), ("status", 3)]);
}

#[test]
fn source_without_a_shebang_is_not_zsh() {
    assert!(!has_shebang("status=1\n"));
    assert!(has_shebang("#!/bin/zsh\nstatus=1\n"));
}
