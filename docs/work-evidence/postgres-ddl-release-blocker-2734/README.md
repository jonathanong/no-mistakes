<!-- cspell:ignore Clippy -->
# PostgreSQL DDL release blocker: issue 2734, section 1

This change covers only the PostgreSQL migration regressions. The other issue
sections and all previously queued follow-ups are outside this PR.

Dollar-quoted bodies already remain atomic during outer statement splitting.
The failure was in procedural recovery: SQL-bearing commands and guarded utility
DDL were not recognized consistently after a body was opened. Saved fixtures
cover DO, function, and procedure commands, utility DDL after BEGIN and IF, and
malformed siblings at their own source lines.

The parser has no unsupported-versus-malformed error category. Complete selected
unsupported grammars therefore carry an explicit classification from full
tokens. These forms advise by default; unrecognized or malformed forms report.
Explicit rule and repository policies override both classes. See the
[migration note](../../migrations/postgres-schema-unanalyzable.md) for the exact
recognizer scope and compatibility AST limitations.

## Validation

The full workspace scope passed 13,448 library tests, 458 binary tests, and
1,402 integration tests, with three existing library ignores. After two
mechanical Clippy cleanups, final validation passed 3,370 PostgreSQL library
tests and 578 compiled native API tests in 21 files. The native cases compare
CommonJS, ESM, and prepared project reports. All-target Clippy, formatting,
duplicate checks, AST guards, and the repository check passed.

PostgreSQL 18 accepted the saved top-level and guarded DDL fixtures, plus all
listed SQL-bearing commands inside DO/function/procedure bodies. The oracle
uses fixture setup files and rolled-back transactions in a private cluster.

Global coverage cleared the 98% floors. Final changed-line coverage is
563/567 (99.29%); no filenames were excluded. Full-scope coverage precedes the
mechanical cleanup; fresh final-source counters refresh the two edited files
and augment unchanged sources. All 15,645 analyzed functions pass CRAP 30.
The 15GB full-scope target was removed immediately after exporting evidence.

## Matched fixture comparison

The baseline is `c94f1b7939d5c234c2f3409440cd3f199a08bcc2`. The same final fixture
inputs were checked by distinct frozen baseline and production candidate CLIs.
Of 1,074 configuration/root pairs, 1,069 have identical stdout, stderr, and exit
status. The five changes are the new positive DDL, guarded DDL, default policy,
and advisory suppression configurations. There are 35 identical pre-existing
configuration failures; those comparisons do not prove successful analysis.
The PostgreSQL-backed integration scope runs separately with PostgreSQL 18.

[validation.json](validation.json) records counts, the exact changed pairs,
binary hashes, production source hashes, and coverage scope. Reproduce the
matched comparison from the repository root with distinct binaries:

```sh
python3 docs/work-evidence/postgres-ddl-release-blocker-2734/compare.py \
  /path/to/baseline /path/to/candidate --out /tmp/postgres-ddl-comparison
```

Use `check-patch-coverage.py` with exported LCOV to check measured Rust diff
lines without excluding filenames. The normal hosted coverage job remains the
final check of the PR head.
