<!-- cspell:ignore Clippy -->
# Boolean keys and RETURNING CTE bounds

Baseline: `aa17b86a2`, Rust 1.99.1. A baseline CLI was built before implementation
and frozen separately from the candidate. Both executables ran `check --root
<saved fixture> --config <name> --format json` on identical saved inputs under
`test-cases/rules/postgres-bounded-statements/fixture`.

| Configuration | Baseline findings | Candidate findings |
| --- | ---: | ---: |
| `boolean-keys.yml` | 14 | 9 |
| `cte-returning-positive.yml` | 6 | 0 |
| `cte-returning-negative.yml` | 7 | 8 |
| `boolean-cte-embedded.yml` | 4 | 1 |

The boolean controls retain non-key, partial composite-key, OR, and NOT findings.
The CTE positives cover UPDATE, DELETE, SELECT, explicit CTE column aliases,
nested SELECT projections, nested WITH, independently bounded target rows with
an unbounded auxiliary input, and an unused write without RETURNING. The embedded
TypeScript case requires both fixes and retains one genuine non-key scan.

The extra negative finding is intentional. An inner `WITH shadows AS (...)`
previously borrowed the key of a physical catalog namesake. Its actual CTE can
supply many distinct target keys. Both the write and its outer join must remain
unbounded. The fixture selects varying `source_id` values and returns varying
physical target IDs, so a fixed comparison on the CTE's separate `id` label
cannot accidentally make the control one distinct key.

All other 17 saved rule configurations retain byte-identical stdout and stderr
and the same exit status. [The comparison](check-comparison.json) records output
hashes and statuses for all 21 configurations.

Validation:

- Locked all-features workspace build includes libraries, binaries and integration
  suites through `.github/scripts/rust-validation.sh build`.
- 3,616 PostgreSQL, rule, prepared-check and N-API project tests pass; two existing
  ignored tests remain ignored. All 431 PostgreSQL CLI suite tests pass.
- N-API standalone and aggregate checks have identical complete reports for all
  four configurations above. The new TypeScript fixture passes `resolve-check`.
- Focused LLVM coverage runs all 224 bounded-statements rule tests. Modified DML
  and CTE scope code and the extracted source-merge helper have 100% line coverage;
  the pin extractor has 98.53%. [The focused report](focused-coverage.json) retains
  line, function and region totals. No filenames were excluded from coverage.
- Clippy across all targets/features, the complexity gate, ast-grep, formatting,
  duplication and spelling checks pass. The repository check has no findings
  or warnings.

No `query_annotation` or `relative_scope` files are changed. No public option or
report shape is added: CLI and asynchronous Node checks share the corrected
prepared facts.
