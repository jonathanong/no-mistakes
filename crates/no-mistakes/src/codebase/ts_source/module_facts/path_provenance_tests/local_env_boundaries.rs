use super::*;

#[test]
fn immediate_env_reads_require_a_single_parameterless_returning_function() {
    let read = read("3304-local-env.mts");
    for variable in [
        "DIRECT",
        "COALESCED",
        "FALLBACK",
        "PARAMETER",
        "REST",
        "GENERATOR",
        "MULTIPLE",
        "CONCISE",
        "BLOCKED",
        "NOT_RETURNED",
    ] {
        let needle = format!("readdirSync(process.env.{variable}");
        let offset = read.source.find(&needle).unwrap() as u32;
        assert_eq!(
            read.provenance().immediate_env_reads.contains(&offset),
            matches!(variable, "DIRECT" | "COALESCED" | "FALLBACK"),
            "{variable}"
        );
    }
    for needle in ["readdirSync()", "readdirSync('fixtures')"] {
        assert!(!read
            .provenance()
            .immediate_env_reads
            .contains(&(read.source.find(needle).unwrap() as u32)));
    }
}

#[test]
fn test_local_env_adjacency_proves_only_named_zero_argument_reads_after_known_stubs() {
    let read = read("3304-local-env.mts");
    for (needle, present) in [
        ("directRead();", true),
        ("functionRead();", true),
        ("conciseRead()", false),
        ("interruptedRead();", false),
        ("argumentRead(", false),
        ("object.memberRead()", false),
        ("wrongMethodRead();", false),
        ("wrongObjectRead();", false),
        ("computedRead();", false),
        ("assignmentRead();", false),
        ("shadowedRead();", false),
    ] {
        let offset = read.source.find(needle).unwrap() as u32;
        assert_eq!(
            read.provenance().test_env_calls.contains_key(&offset),
            present,
            "{needle}"
        );
        if let Some(stub_end) = read.provenance().test_env_calls.get(&offset) {
            assert!(*stub_end < offset);
            assert_eq!(&read.source[*stub_end as usize..offset as usize], "; ");
        }
    }
    assert_eq!(read.provenance().test_env_calls.len(), 2);
}
