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
    let source = std::fs::read_to_string(&path).unwrap();
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(&path).unwrap();
    let parsed = Parser::new(&allocator, &source, source_type).parse();
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
    assert!(
        read("stub-global-chains.mts")
            .provenance()
            .env_unknown_write
    );
}

mod call_origins;
mod member_arguments;

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

mod argument_text;
mod member_origins;
mod temp_factories;
mod tmpdir;

mod captured_wrappers;

mod getters;

mod lifecycle_boundaries;

mod captured_wrapper_edges;

mod getter_import_kinds;

mod callback_roots;
mod env_diagnostic;
