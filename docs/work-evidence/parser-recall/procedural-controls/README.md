<!-- cspell:ignore Clippy -->
# Procedural recovery controls

This review delta extends the original parser-recall change to recognized
PL/pgSQL controls in DO, function and procedure bodies. Complete wrapper
validation includes the routine header and EOF; malformed headers and suffixes
remain reported. The PROCEDURE keyword is normalized only for parser proof;
retained SQL facts keep their original kind.

Saved fixtures cover nested IF/ELSE, CASE, LOOP, labels, EXIT, CONTINUE, RETURN,
RAISE and PERFORM, plain/E-quoted DO, dollar-quoted functions/procedures, unknown
first actions, malformed routine headers and failed procedural SQL payloads.
Valid bodies are clean. Unknown SQL after each body and stray top-level END IF
keep their own source lines. Static EXECUTE recovery is unchanged.

The baseline is reviewed head `7678f9205` rebased onto `69ba98b7` as `777d3cc5f`;
parser and migration source trees are identical across that rebase. The final
pre-push candidate is `9b62dca90`. Both frozen CLI builds use the same locked
default-feature build command, Rust 1.99.1 and disabled incremental compilation.
[Executable hashes and comparisons](check-comparison.json) record all 109 saved
configurations. Of those, 107 have identical stdout, stderr and exit status.

| Configuration | Before findings | After findings |
| --- | --- | --- |
| Procedural recovery corpus | 38 | 14 |
| Existing identifier-length failure corpus | 24 | 22 |

The procedural corpus drops all 20 markers from valid.sql and both quoted-DO
wrapper markers, while preserving 14 real unknown statements. It also recovers
a previously lost first action after IF. The identifier-length change removes
two duplicate procedure-wrapper diagnostics; definite long-name findings stay.
All definite statement-policy violations remain checked by the existing tests.

Validation:

- Canonical locked all-features Rust CI scope at `ffaf8603d`: 13,286 library,
  449 binary and 1,389 integration tests pass; three existing tests are ignored.
- All 575 compiled native tests pass across the 21 Linux CI files, including
  CJS/ESM/project parity for the expanded procedural corpus.
- All 2,154 focused tests pass after the pure token/whitespace helper extraction;
  two existing tests are ignored. The explicit Node report parity test passes.
- Focused LLVM validation covers the same 2,154 cases without filename
  exclusions. [Helper totals and source hashes](focused-coverage.json) cover the
  unchanged marker helpers before the final token-only module extraction.
- Clippy, formatting, spelling, ast-grep, duplication and complexity pass. The
  final repository check has no findings, warnings or advisories.

The existing lenient parser's 227 physical lines are reduced to 192 by moving
unchanged token chunk/whitespace helpers to a sibling module. All touched Rust
sources meet 200 physical lines and tests meet 500. Body wrapper markers are
removed in one grouped pass rather than scanning diagnostics per fragment.
No query_annotation or relative_scope paths are edited. Both worker worktrees
and the focused coverage target are removed.

The final branch is rebased onto `17139a03b` as `a36e23002`. Parser, migration
and native parity helper trees are unchanged by that rebase. Main includes an
unrelated Rust annotation change, so the canonical scope is repeated: 13,287
library tests (three existing ignored), 449 binary tests and 1,389 integration
tests pass. All 575 rebuilt native tests pass again. All 109 CLI outputs match
the pre-rebase candidate, and the repository check remains clean. The comparison
artifact records the rebased executable hash separately.

The required Codecov Rust patch status at `b62a815d5` measured 97.84%, below its
98% target. Local checks had validated focused/global coverage without measuring
the full PR diff. Redundant pre-parser header guards were removed: the complete
CREATE FUNCTION parser result and EOF check already reject malformed headers.
The existing negative fixtures remain passing.

Full workspace LLVM coverage now passes all unchanged 98% line/function/region
floors. [Local measured diff coverage](patch-coverage.json) is 679/692 lines,
98.1214%, against the same 98% patch target. No filenames are excluded. The
coverage target exceeded 10 GB and was removed immediately after export.

Before another push, export full LCOV and run the saved checker after staging
new Rust files:

```sh
python3 docs/work-evidence/parser-recall/procedural-controls/check-patch-coverage.py /path/to/full-coverage.lcov --base origin/main --minimum 98
```

The checker reports measured changed lines, misses, and files without executable
line records. It reads the working tree relative to the PR merge base and adds
no analyzer state or runtime cache.
