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
