# Shared-prefix memoization at b830343a6

Completed immutable shared records retain their direct prefix writes for the current
collection. Revisits replay those writes in last-writer order. Argument-derived
writes retain first-visit semantics; unfinished records reached through cycles are
not cached. Before and after collections have separate caches and argument arenas.
Unique records forward writes without retaining overlapping ancestor summaries.

| Saved depth-12 fixture | Original walk | Memoized walk |
| --- | ---: | ---: |
| Visited values | 131,012 | 788 |
| Container visits | 65,476 | 676 |
| Comparisons | 30 | 30 |

The complete JSON reports are identical. Saved 30-level prefix and argument-leaf
DAG tests bound visits by twice their 31 records plus two leaf visits. Frozen
legacy-map comparisons cover overwrite order, cycles, sparse/deleted slots,
separate arenas, container variants, and a 128-record distinct-prefix chain.
This eliminates repeated expansion of shared DAGs; it does not establish universal
linear complexity for every possible prefix inventory or snapshot shape.

Exact-source validation at `b830343a6a3809e1d37df5fecb38931586337810` passes:

- 13,809 workspace tests; three existing ignored tests.
- 99.3329% lines, 99.3278% functions, 98.5172% regions; all 98% floors pass.
- 272/272 changed executable lines; both CRAP gates pass.
- Rust linting with warnings denied, documentation tests, formatting, configured checks,
  duplicate analysis, and fixture-document spelling pass.

Every configured Postgres fixture root compares identically: 523 complete reports
and five intentional configuration errors across 528 roots. The two fixtures that
require live-generated catalogs were also compared after private PostgreSQL
catalog generation. `fixture-rows.json` records the original per-root report hashes;
`generated-catalog-fixtures.json` supplies the two generated-catalog reports.
The complete workspace golden suite covers fixtures without their own config.

The baseline CLI was built at `633724ce9`; its analyzer source tree is identical
to published `e106b3d15`, as recorded in `work-and-coverage.json`. The package-version
and documentation rebase is not represented as a fresh baseline sample. Candidate
and baseline native correctness binaries use the same all-feature debug build.
These comparisons establish work bounds and recall, not timing or RSS gains.

Full current consumer CPU recovery and diagnostic overhead are still being
measured. Both 120-second candidate diagnostic preflight runs were censored during
concurrent compilation and cannot support a performance comparison. A fresh
20-second post-memo stack sample still finds deep captured-binding snapshot clones.
No full-consumer recovery, adoption, or completed RSS attribution is claimed here.
All Cargo targets and private PostgreSQL resources used for these gates were deleted.
