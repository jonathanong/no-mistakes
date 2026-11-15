use super::super::coverage::{assemble, blank, call_site, empty_graph};
use super::branch::{covering_line, external_branch, file_hit, Cover, FileHit};
use super::*;
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::{
    ConditionalPlace, EnvWrite, PathProvenance, WriteScope,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct Facts(FxHashMap<PathBuf, TsFileFacts>);
impl TsFactLookup for Facts {
    fn get_ts_facts(&self, file: &Path) -> Option<&TsFileFacts> {
        self.0.get(file)
    }
}

fn hint(name: &str) -> EnvDiagnostic {
    EnvDiagnostic {
        name: name.to_string(),
        fallback: Some("./tmp/example".into()),
        captured: false,
        line: 3,
    }
}

#[test]
fn ambiguous_getter_diagnostics_do_not_choose_one_environment_variable() {
    let mut facts = Facts(FxHashMap::default());
    for (file, name) in [("/repo/a.mts", "ROOT_A"), ("/repo/b.mts", "ROOT_B")] {
        let mut origin = PathOrigin::unknown();
        origin.env = Some(name.into());
        origin.env_diagnostic = Some(Box::new(hint(name)));
        let mut paths = PathProvenance::default();
        paths.env_getters.insert(4, origin);
        facts.0.insert(
            file.into(),
            TsFileFacts {
                module_bindings: Some(Arc::new(TypeScriptModuleFacts {
                    path_provenance: Some(Arc::new(paths)),
                    ..TypeScriptModuleFacts::default()
                })),
                ..TsFileFacts::default()
            },
        );
    }
    let mut first = call_site(Path::new("/repo/query.mts"), "query", 1);
    first.target_node = Some(NodeId::callable("/repo/a.mts", "root", CallableId(4)));
    let mut second = first.clone();
    second.target_node = Some(NodeId::callable("/repo/b.mts", "root", CallableId(4)));
    assert_eq!(
        getter_environment_hint(&first, &facts).unwrap().name,
        "ROOT_A"
    );
    assert_eq!(
        getter_environment_hint(&second, &facts).unwrap().name,
        "ROOT_B"
    );
    first.alternatives.push(second);
    assert!(getter_environment_hint(&first, &facts).is_none());
    let graph = empty_graph();
    let owners = assemble(blank(&graph));
    let mut origin = PathOrigin::unknown();
    origin.getter = Some(first.offset);
    assert!(matches!(
        owners.ownership(&origin, &first, &facts, &[]),
        super::super::Ownership::No
    ));
}

#[test]
fn an_unknown_entry_does_not_name_a_production_location_as_the_test_culprit() {
    let graph = empty_graph();
    let owners = assemble(blank(&graph));
    let facts = Facts(FxHashMap::default());
    let mut site = call_site(Path::new("/repo/production.mts"), "scan", 1);
    site.line = 7;
    let mut origin = PathOrigin::unknown();
    origin.env_diagnostic = Some(Box::new(hint("ROOT")));
    for _ in 0..2 {
        let cause = owners.env_cause(&origin, &site, &facts).unwrap();
        assert!(!cause.concrete);
        assert_eq!(cause.file, site.file);
        assert_eq!(cause.line, site.line);
        let message = cause.message(Path::new("/repo"), "readdirSync");
        assert!(message.contains("process.env.ROOT"));
        assert!(message.contains("literal fallback \"./tmp/example\""));
        assert!(message.contains("a concrete test or setup entry cannot be identified"));
        assert!(message.contains("make the test-to-scan call or import route explicit"));
        assert!(!message.contains("production.mts:7"));
        assert!(!message.contains("clears that environment variable"));
    }
}

#[test]
fn a_known_static_import_remains_a_concrete_location_without_a_call_offset() {
    let cause = EnvCause {
        hint: hint("ROOT"),
        file: "/repo/test.mts".into(),
        line: 4,
        reason: "has no proven assignment before module initialization",
        concrete: true,
        branch: None,
    };
    let message = cause.message(Path::new("/repo"), "readdirSync");
    assert!(message.contains("test.mts:4 has no proven assignment before module initialization"));
    assert!(message.contains("Fix: at test.mts:4"));
}

#[test]
fn a_module_target_cannot_identify_an_environment_getter() {
    let mut site = call_site(Path::new("/repo/query.mts"), "query", 1);
    site.target_node = Some(NodeId::file("/repo/getter.mts"));
    let facts = Facts(FxHashMap::default());
    assert!(getter_environment_hint(&site, &facts).is_none());
}

#[test]
fn unreadable_or_unparsed_facts_cannot_supply_environment_diagnostics() {
    let bindings = Arc::new(TypeScriptModuleFacts {
        path_provenance: Some(Arc::new(PathProvenance::default())),
        ..TypeScriptModuleFacts::default()
    });
    let valid = TsFileFacts {
        module_bindings: Some(bindings.clone()),
        ..TsFileFacts::default()
    };
    assert!(diagnostic_paths(&valid).is_some());
    for (operational_error, parse_error, fatal_parse_error) in [
        (Some("read failed".to_owned()), None, false),
        (None, Some("parse failed".to_owned()), false),
        (None, None, true),
    ] {
        let failed = TsFileFacts {
            module_bindings: Some(bindings.clone()),
            operational_error,
            parse_error,
            fatal_parse_error,
            ..TsFileFacts::default()
        };
        assert!(diagnostic_paths(&failed).is_none());
    }
}

#[test]
fn missing_entry_facts_keep_the_known_test_location_without_inventing_a_write() {
    let graph = empty_graph();
    let owners = assemble(blank(&graph));
    let facts = Facts(FxHashMap::default());
    let entry = Entry {
        file: "/repo/test.mts".into(),
        offset: Some(42),
        line: 7,
        concrete: true,
    };
    let (rank, cause, offset) = owners.rank_environment_entry(
        entry,
        hint("ROOT"),
        "has no proven assignment before reaching this scan",
        &facts,
    );
    assert_eq!(rank, 4);
    assert_eq!(offset, Some(42));
    assert!(cause.concrete);
    assert_eq!(cause.line, 7);
    let message = cause.message(Path::new("/repo"), "readdirSync");
    assert!(message.contains("test.mts:7 has no proven assignment"));
    assert!(!message.contains("clears that environment variable"));
    assert!(!message.contains("only inside a conditional"));
}

fn env_write(name: &str, line: u32, end: u32, scope: WriteScope) -> EnvWrite {
    EnvWrite {
        line,
        cleared: false,
        name: name.into(),
        end,
        origin: PathOrigin::unknown(),
        scope,
    }
}

fn branch_place(
    branch: bool,
    hook: Option<(u32, u32)>,
    function: Option<(u32, u32)>,
) -> WriteScope {
    WriteScope::Conditional(ConditionalPlace {
        hook,
        function,
        program: hook.is_none() && function.is_none(),
        branch,
    })
}

fn provenance(writes: Vec<EnvWrite>, callbacks: Vec<(u32, u32)>) -> PathProvenance {
    PathProvenance {
        env_writes: writes,
        test_callbacks: callbacks,
        ..PathProvenance::default()
    }
}

#[test]
fn one_conditional_assignment_names_the_test_and_that_branch() {
    let paths = provenance(
        vec![env_write(
            "ROOT",
            6,
            40,
            branch_place(true, Some((0, 400)), None),
        )],
        vec![(100, 200)],
    );
    assert!(matches!(covering_line(&paths, "ROOT", 150), Cover::One(6)));
    let outside = provenance(
        vec![env_write(
            "ROOT",
            6,
            40,
            branch_place(true, Some((0, 50)), None),
        )],
        vec![(100, 200)],
    );
    assert!(matches!(covering_line(&outside, "ROOT", 150), Cover::None));
    let program = provenance(
        vec![env_write("ROOT", 1, 10, branch_place(true, None, None))],
        vec![],
    );
    assert!(matches!(covering_line(&program, "ROOT", 20), Cover::One(1)));
    assert!(matches!(covering_line(&program, "ROOT", 5), Cover::None));
    let graph = empty_graph();
    let owners = assemble(blank(&graph));
    let facts = facts_for("/repo/test.mts", paths);
    let (rank, cause, _) = owners.rank_environment_entry(
        Entry {
            file: "/repo/test.mts".into(),
            offset: Some(150),
            line: 9,
            concrete: true,
        },
        hint("ROOT"),
        "has no proven assignment before reaching this scan",
        &facts,
    );
    assert_eq!(rank, 4);
    let message = cause.message(Path::new("/repo"), "readdirSync");
    assert!(message.contains("test.mts:9 reaches this scan"));
    assert!(message.contains("test.mts:6 assigns `process.env.ROOT` only inside a conditional"));
    assert!(message.contains("at test.mts:6, assign \"ROOT\" on every path"));
}

#[test]
fn an_expression_write_or_two_branches_are_not_named_as_the_unset_branch() {
    let expression = provenance(
        vec![env_write("ROOT", 3, 20, branch_place(false, None, None))],
        vec![],
    );
    assert!(matches!(
        covering_line(&expression, "ROOT", 30),
        Cover::None
    ));
    assert!(matches!(file_hit(&expression, "ROOT"), FileHit::Blocked));
    let doubled = provenance(
        vec![
            env_write("ROOT", 4, 20, branch_place(true, None, Some((0, 80)))),
            env_write("ROOT", 5, 30, branch_place(true, None, Some((0, 80)))),
        ],
        vec![],
    );
    assert!(matches!(covering_line(&doubled, "ROOT", 40), Cover::Many));
    let unrelated = provenance(
        vec![env_write(
            "ROOT",
            2,
            15,
            branch_place(true, None, Some((0, 20))),
        )],
        vec![(40, 80)],
    );
    assert!(matches!(covering_line(&unrelated, "ROOT", 50), Cover::None));
    let late = provenance(
        vec![env_write(
            "ROOT",
            4,
            50,
            branch_place(true, None, Some((0, 80))),
        )],
        vec![],
    );
    assert!(matches!(covering_line(&late, "ROOT", 40), Cover::None));
    let plain = provenance(vec![env_write("ROOT", 1, 4, WriteScope::Program)], vec![]);
    assert!(matches!(covering_line(&plain, "ROOT", 10), Cover::None));
    let graph = empty_graph();
    let owners = assemble(blank(&graph));
    let facts = facts_for("/repo/test.mts", expression);
    let (_, cause, _) = owners.rank_environment_entry(
        Entry {
            file: "/repo/test.mts".into(),
            offset: Some(30),
            line: 8,
            concrete: true,
        },
        hint("ROOT"),
        "has no proven assignment before reaching this scan",
        &facts,
    );
    let message = cause.message(Path::new("/repo"), "readdirSync");
    assert!(message.contains("test.mts:8 has no proven assignment"));
    assert!(!message.contains("only inside a conditional"));
}

#[test]
fn one_external_conditional_is_named_and_a_second_blocks_it() {
    let setup = provenance(
        vec![env_write("ROOT", 5, 10, branch_place(true, None, None))],
        vec![],
    );
    let other = provenance(
        vec![env_write("ROOT", 2, 8, branch_place(true, None, None))],
        vec![],
    );
    assert!(matches!(file_hit(&setup, "ROOT"), FileHit::Branch(5)));
    assert!(matches!(
        file_hit(&PathProvenance::default(), "ROOT"),
        FileHit::Absent
    ));
    let blocked = provenance(vec![env_write("ROOT", 1, 4, WriteScope::Program)], vec![]);
    assert!(matches!(file_hit(&blocked, "ROOT"), FileHit::Blocked));
    let mut facts = facts_for("/repo/setup.mts", setup);
    facts.0.insert("/repo/other.mts".into(), stored(other));
    facts.0.insert("/repo/blocked.mts".into(), stored(blocked));
    let named = external_branch(&[PathBuf::from("/repo/setup.mts")], "ROOT", &facts).unwrap();
    assert_eq!(named.1, 5);
    assert!(external_branch(
        &[
            PathBuf::from("/repo/setup.mts"),
            PathBuf::from("/repo/other.mts")
        ],
        "ROOT",
        &facts,
    )
    .is_none());
    assert!(external_branch(&[PathBuf::from("/repo/blocked.mts")], "ROOT", &facts).is_none());
    assert!(external_branch(&[PathBuf::from("/repo/missing.mts")], "ROOT", &facts).is_none());
}

fn facts_for(file: &str, paths: PathProvenance) -> Facts {
    let mut facts = Facts(FxHashMap::default());
    facts.0.insert(file.into(), stored(paths));
    facts
}

fn stored(paths: PathProvenance) -> TsFileFacts {
    TsFileFacts {
        module_bindings: Some(Arc::new(TypeScriptModuleFacts {
            path_provenance: Some(Arc::new(paths)),
            ..TypeScriptModuleFacts::default()
        })),
        ..TsFileFacts::default()
    }
}
