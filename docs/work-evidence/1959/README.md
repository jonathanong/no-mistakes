# Nested imported receiver writes: work evidence

Issue: <https://github.com/jonathanong/no-mistakes/issues/1959>.
Source base: `917c9ea1c2f2400e8a5d3d50c18c0a52fe21ad4a`.

The saved fixture is
`test-cases/rules/query-reached-per-item/provenance-fixture/src/nested-receiver-writes.mts`.
Its graph test checks member precision, another imported receiver, method
snapshots, replaced ancestor references, mutable aliases, and acquisition timing.
The extraction fixtures also check canonical history sharing and bounded work.

## Baseline probes

The authoritative graph probe used the existing `receiver-current-targets`
harness with the fixture root and `src/nested-receiver-writes.mts` arguments.
Its SHA-256 is
`1fa11a4991c27a7de8637faece3842e23e17285c64f05a8409dead6046ab4646`.
The harness was built from `5b8710b4e0b044fd35b3c770332131ccb857e1b4`;
its production source and Cargo.lock were verified equal to the source base.
[Build metadata](baseline-build.json) and the complete, unmodified
[graph output](baseline-targets.txt) are retained here.

An additional existing CLI probe ran:

```sh
no-mistakes check --root <fixture-root> --format json
```

Its exit status was 1, and its SHA-256 was
`c9bf1601755eaef2addd6f521459019b39c5e66d41f37cf47fa2703816108ad0`.
The CLI's exact source revision is not established; it is supporting evidence,
not an authoritative revision comparison. Its complete, unmodified
[JSON output](baseline-cli.json) includes findings at lines
8, 12, 16, 18, 20, 22, 41, 47, 55, 64, 73, 81, 89, 98, and 107 of the saved file.

## Conservative context handling

The verified baseline reports `primary.get` for all three controls below.
The candidate tests require `Unknown`; candidate execution remains pending.

| Helper | Loop line | Reason for the conservative result |
| --- | ---: | --- |
| `calledReceiver` | 89 | A different callable replaces the ancestor reference before the nested write. |
| `mixedReceiver` | 98 | The replacement summary combines writes from different callable contexts. |
| `inverseReceiver` | 107 | The nested write precedes the replacement callable's invocation. |

The existing facts do not establish execution order across callable contexts.
Only an ordered replacement in the same context can sever the initializer's
receiver path. With uncertain ordering, the candidate retains the possible
mutation of the original receiver instead of claiming its imported method is
unchanged. This deliberately loses the baseline's imported-effect proof for
the first two controls; the inverse control prevents stale provenance after an
earlier mutation. This is not general heap or call execution modeling.

## Validation status

Local compilation is paused for coordinated performance work. Semantic tests,
all 70 existing receiver/default/inline controls, the full graph suite, Rust lint checks,
coverage, complexity checks, and matched performance measurements remain
required before the draft is ready. Work-count assertions demonstrate bounded
sharing, not a measured CPU or memory improvement. Introduced regressions and
material performance regressions block readiness.

Before draft publication, AST rule tests (17), the AST scan, changed Rust
format checks, JavaScript formatting/lint, documentation spelling, and the
repository's configured checks passed. The repository check used the supplied
prebuilt CLI and returned no findings, warnings, or advisories; it did not
execute the candidate Rust implementation. Duplicate-source checks and the
complete pre-push Rust checks remain pending.

## Shepherd Journal

- The initial exact-write propagation consumed the assigned member at the
  syntactic receiver. Retaining it for nested identities fixes the issue's
  path, while ancestor-reference cutoffs protect replaced references and
  previously acquired receivers.
- Static review identified inverse callable execution order as a stale-target
  risk. The candidate keeps uncertain mutation evidence; the precision loss
  above is explicit and must be reviewed before readiness.
- Repeated ancestor checks originally walked the alias component for each
  write. A request-owned union-by-size index now keeps one mutable history per
  component and produces shared final histories. Saved operation-count tests
  cover aliases, repeated writes, and unrelated bindings.
