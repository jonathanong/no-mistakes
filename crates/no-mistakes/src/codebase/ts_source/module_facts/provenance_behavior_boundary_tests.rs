use super::*;
use arguments::{PathAnchor, PathOrigin};

struct Fixture {
    source: String,
    facts: TypeScriptModuleFacts,
}

fn fixture(name: &str) -> Fixture {
    let path =
        crate::test_support::rule_fixture_root("test-no-live-repository-scan/facts").join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        extract_with_arguments(program, true)
    })
    .unwrap();
    assert!(
        facts.call_arguments.is_some(),
        "{name}: {:?}",
        facts.diagnostics
    );
    Fixture { source, facts }
}

impl Fixture {
    fn offset(&self, case: &str, callee: &str) -> u32 {
        let line = self
            .source
            .lines()
            .find(|line| line.contains(&format!("'{case}'")))
            .unwrap();
        (self.source.find(line).unwrap() + line.find(callee).unwrap()) as u32
    }

    fn origin(&self, case: &str) -> &PathOrigin {
        self.facts.call_arguments.as_ref().unwrap()[&self.offset(case, "readdirSync(")]
            .path
            .as_ref()
            .unwrap()
    }
}

#[test]
fn host_reads_and_require_resolution_preserve_env_proofs_but_escapes_poison_them() {
    for (name, case, poisoned) in [
        ("3304-prov-safe-host.mjs", "safe-host", false),
        ("3304-prov-require-resolve.cjs", "require-resolve", false),
        ("3304-prov-require-cache.cjs", "require-cache", true),
        ("3304-prov-require-escape.cjs", "require-escape", true),
        ("3304-prov-env-comparison.mjs", "env-comparison", true),
    ] {
        let read = fixture(name);
        let provenance = read.facts.path_provenance.as_ref().unwrap();
        assert_eq!(provenance.env_unknown_write, poisoned, "{name}");
        assert_eq!(provenance.env_opaque_write, poisoned, "{name}");
        assert_eq!(read.origin(case).env.as_deref(), Some("ROOT"), "{name}");
    }
}

#[test]
fn owned_helpers_need_every_return_to_stay_below_the_owned_directory() {
    let read = fixture("3304-prov-owned-returns.mjs");
    for case in ["both", "loop"] {
        let origin = read.origin(case);
        assert_eq!(origin.anchor, PathAnchor::Temp, "{case}: {origin:?}");
        assert!(
            origin
                .atoms()
                .iter()
                .all(|origin| origin.anchor == PathAnchor::Temp
                    && !origin.segments.iter().any(|part| part == "..")),
            "{case}: {origin:?}"
        );
    }
    for case in ["parent", "incomplete", "unsupported", "expression"] {
        let origin = read.origin(case);
        assert_eq!(origin.anchor, PathAnchor::Unknown, "{case}: {origin:?}");
    }
}

#[test]
fn member_paths_keep_disjoint_reads_but_reject_escaping_or_mutated_roots() {
    let read = fixture("3304-prov-member-effects.mjs");
    let sibling = read.origin("sibling-read");
    assert_eq!(sibling.param, Some(0), "{sibling:?}");
    assert_eq!(
        sibling.param_member.as_deref(),
        Some("paths.root"),
        "{sibling:?}"
    );
    assert!(!sibling.require_scalar, "{sibling:?}");
    for case in [
        "literal-key",
        "root-escape",
        "reassigned",
        "unknown-root",
        "dynamic-read",
        "mutate-root",
    ] {
        let origin = read.origin(case);
        assert_eq!(origin.anchor, PathAnchor::Unknown, "{case}: {origin:?}");
        assert!(origin.param.is_none(), "{case}: {origin:?}");
    }
    let effects = &read
        .facts
        .path_provenance
        .as_ref()
        .unwrap()
        .arg_member_effects[&read.offset("literal-effects", "inspect(")];
    assert_eq!(effects.len(), 1, "{effects:?}");
    assert_eq!(effects[0].0, 0);
    assert_eq!(effects[0].1.path, ["root"]);
    assert!(effects[0].1.escaped);
}
