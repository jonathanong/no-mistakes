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

## First draft CI and review corrections

At `b1f2cc155d32110df3d10156f24f41669b3af76f`, CI passed formatting,
Rust lint, duplication, spelling, AST checks, JavaScript, and both native
platform jobs. The Rust library run passed 11,243 tests and failed two existing
default-origin tests: an untouched literal-star selection lost its proof, and
copied-method/replaced-container controls lost findings at lines 16 and 19.
Those expectations remain unchanged.

The correction separates newly propagated exact assignments from direct
assignments inside the existing optional assignment box. Supplied receiver
reads use the aggregate summary. Selected default reads retain their Source
prefix evidence and use their owned writer's propagated summary, preserving
acquisition offsets without importing unrelated helper writers. This adds
optional writer metadata; its cold memory cost still requires measurement.

The conditional-reference review regression is saved in
`nested-receiver-cutoffs.mts`. Its [raw verified baseline output](conditional-baseline-targets.txt)
retains `primary.get` for every helper. Candidate tests require Unknown except
for the guaranteed fresh replacement control. Conditional branches, loops,
short-circuit writes, logical assignments, default right-hand sides,
self-assignment, and unknown values cannot establish a fresh receiver identity.
The earliest guaranteed fresh replacement offset is stored separately from
the earliest possible write. A saved receiver acquired between a conditional
write and a later guaranteed replacement retains its earlier identity.

The traversal counter now measures actual existing alias-edge propagation:
128 writes over 128 undirected links assert 32,768 directed edge visits. That
baseline traversal is still proportional to writes times alias edges. The
canonical root counter separately bounds the additional ancestor checks; only
their redundant component walk was eliminated. No linear whole-collector or
measured performance claim is made. These corrections await current-head CI.

The correction's next CI run passed 11,247 library tests, including all existing
receiver/default graph and rule controls, but exposed two fact guards: the
unchanged 16-byte exact-summary bound and a binding-default traversal bypass.
The next correction packs module/single/mixed writer state into one enum and
stores the guaranteed nonzero completion offset inline, preserving the bound.
Both object and array binding-default paths now retain receiver-only conditional
certainty in the existing named-default visitor. No semantic expectations were
relaxed. Current-head runtime validation remains pending.
