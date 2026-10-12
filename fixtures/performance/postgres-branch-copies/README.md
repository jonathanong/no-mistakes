# PostgreSQL branch-state copy benchmark

Twelve entry modules call one helper with 64 literal arguments. Its two
branches retain callback references to a cyclic `arguments` object while most
dense slots remain unchanged. Each entry checks one annotated query and one
unannotated query, so the expected report has twelve findings and no warnings.
The helper's saved `retained` value lets a Rust test assert callback and cyclic
argument reachability directly, beyond the query findings.
The fixture is saved on disk and is shared unchanged by baseline and candidate
benchmarks.
