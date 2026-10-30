use super::arguments::{PathAnchor, PathOrigin, PathProvenance, WriteScope};
use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

struct Read {
    source: String,
    facts: TypeScriptModuleFacts,
}

fn read(name: &str) -> Read {
    let path =
        crate::test_support::rule_fixture_root("test-no-live-repository-scan/facts").join(name);
    let source = std::fs::read_to_string(path).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let facts = extract_with_arguments(&parsed.program, true);
    Read { source, facts }
}

impl Read {
    fn provenance(&self) -> &PathProvenance {
        self.facts.path_provenance.as_deref().unwrap()
    }

    /// First argument origin of the call that starts at `needle`.
    fn origin(&self, needle: &str) -> PathOrigin {
        let offset = self
            .source
            .find(needle)
            .unwrap_or_else(|| panic!("missing {needle}")) as u32;
        self.facts.call_arguments.as_ref().unwrap()[&offset]
            .path
            .clone()
            .unwrap_or_else(|| panic!("no path for {needle}"))
    }
}

fn write<'a>(provenance: &'a PathProvenance, name: &str) -> &'a super::arguments::EnvWrite {
    provenance
        .env_writes
        .iter()
        .find(|write| write.name == name)
        .unwrap_or_else(|| panic!("no write of {name}"))
}

#[test]
fn env_writes_record_the_scope_that_must_dominate_a_call() {
    let read = read("env-writes.mjs");
    let provenance = read.provenance();
    assert!(!provenance.env_unknown_write);
    for program in ["A", "B", "C", "D", "E", "G", "L"] {
        assert_eq!(
            write(provenance, program).scope,
            WriteScope::Program,
            "{program}"
        );
    }
    for owned in ["A", "B", "E", "F", "G", "H", "I", "J"] {
        assert_eq!(
            write(provenance, owned).origin.anchor,
            PathAnchor::Temp,
            "{owned}"
        );
    }
    // Compound assignment, delete, and a stub with no value are not owned roots.
    for unowned in ["C", "D", "L"] {
        assert_eq!(
            write(provenance, unowned).origin.anchor,
            PathAnchor::Unknown,
            "{unowned}"
        );
    }
    assert!(matches!(
        write(provenance, "H").scope,
        WriteScope::Function(..)
    ));
    assert!(matches!(
        write(provenance, "J").scope,
        WriteScope::Function(..)
    ));
    assert!(matches!(write(provenance, "I").scope, WriteScope::Hook(..)));
    // A branch, or a write used as an expression, may not have run.
    assert_eq!(write(provenance, "F").scope, WriteScope::Conditional);
    assert_eq!(write(provenance, "K").scope, WriteScope::Conditional);
    assert_eq!(provenance.test_callbacks.len(), 2);
}

#[test]
fn unattributable_env_mutations_are_marked_unknown() {
    for name in [
        "env-unknown-key.mjs",
        "env-delete-unknown.mjs",
        "env-stub-unknown.mjs",
        "env-assign-object.mjs",
        "env-replace.mjs",
        "env-alias.mjs",
        "env-bare-argument.mjs",
    ] {
        let read = read(name);
        assert!(read.provenance().env_unknown_write, "{name}");
    }
    let read = read("env-reads.mjs");
    assert!(!read.provenance().env_unknown_write);
    assert!(read.provenance().env_writes.is_empty());
}

#[test]
fn stub_global_process_and_opaque_config_keys_poison() {
    assert!(
        read("stub-global-process.mjs")
            .provenance()
            .env_unknown_write
    );
    let other = read("stub-global-other.mjs");
    assert!(!other.provenance().env_unknown_write);
    assert!(!other.provenance().opaque_config_key);
    let computed = read("config-computed-key.mjs");
    assert!(computed.provenance().opaque_config_key);
    assert!(!computed.provenance().env_unknown_write);
}

#[test]
fn origins_follow_getters_owned_lets_and_parameters_but_nothing_else() {
    let read = read("origins.mjs");
    let env = |needle: &str| read.origin(needle).env;
    assert_eq!(env("readdirSync(decl())").as_deref(), Some("A"));
    assert_eq!(env("readdirSync(expr())").as_deref(), Some("B"));
    assert_eq!(env("readdirSync(arrow())").as_deref(), Some("C"));
    assert_eq!(env("readdirSync(block())").as_deref(), Some("D"));
    assert_eq!(env("readdirSync(process.env.H").as_deref(), Some("H"));
    for unknown in [
        "readdirSync(withArg(1))",
        "readdirSync(gen())",
        "readdirSync(twice())",
        "readdirSync(missing())",
        "readdirSync(argArrow())",
        "readdirSync(noReturn())",
        "readdirSync(notFunction())",
        "readdirSync(\"x\" && process.env.I)",
        "readdirSync(process.env[String(1)])",
        "readdirSync(process.env.J && \"x\")",
    ] {
        assert_eq!(read.origin(unknown), PathOrigin::unknown(), "{unknown}");
    }
    assert_eq!(read.origin("readdirSync(owned)").anchor, PathAnchor::Temp);
    for unowned in [
        "readdirSync(mixed)",
        "readdirSync(compound)",
        "readdirSync(unset)",
        "readdirSync(exported)",
        "readdirSync(destructured)",
        "readdirSync(pattern)",
    ] {
        assert_eq!(read.origin(unowned), PathOrigin::unknown(), "{unowned}");
    }
    let param = |needle: &str| read.origin(needle).param;
    // Only an unreassigned, plain parameter of the function holding the call.
    assert_eq!(param("readdirSync(first)"), None);
    assert_eq!(param("readdirSync(second);"), Some(1));
    assert_eq!(param("readdirSync(rest)"), None);
    assert_eq!(param("readdirSync(second)]"), None);
    assert_eq!(param("readdirSync(a)"), None);
    assert_eq!(param("readdirSync(b)"), None);
    assert_eq!(param("readdirSync(join(c"), Some(2));
}

#[test]
fn only_owned_or_module_rooted_arguments_of_imported_calls_are_kept() {
    let read = read("origins.mjs");
    let offset = read.source.find("local(\"x\"").unwrap() as u32;
    let paths = &read.provenance().arg_paths[&offset];
    let kept: Vec<(usize, PathAnchor)> = paths
        .iter()
        .map(|(index, origin)| (*index, origin.anchor))
        .collect();
    assert_eq!(
        kept,
        [
            (1, PathAnchor::Temp),
            (3, PathAnchor::Module),
            (4, PathAnchor::Cwd)
        ]
    );
}

#[test]
fn only_exact_rest_forwarding_inside_a_mock_implementation_passes_through() {
    let read = read("pass-through.mjs");
    let forwarded: Vec<usize> = read
        .provenance()
        .pass_through
        .keys()
        .map(|offset| read.source[..*offset as usize].lines().count())
        .collect();
    let mut forwarded = forwarded;
    forwarded.sort();
    // Lines 4, 6, 25 and 33 (1-based) forward exactly; every other shape keeps its
    // advisory, and a bare `vi.fn()` (24) or computed spy name (26) is not recorded.
    assert_eq!(forwarded, [4, 6, 25, 33]);
    let mut names: Vec<&str> = read
        .provenance()
        .pass_through
        .values()
        .map(String::as_str)
        .collect();
    names.sort();
    assert_eq!(names, ["list", "readdirSync", "readdirSync", "readdirSync"]);
}
