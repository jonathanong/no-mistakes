<!-- cspell:ignore clippy -->
# PostgreSQL statement locations: issue 2760

Accepted statements retain their existing AST and token context. Finding locations
use that statement's own span instead of the first matching text or an ordinal
count of lexical keywords. Inline column FKs and SELECT relations keep their
existing declaration anchors so line suppressions continue to work.

The parser, rejected-statement recovery, and unanalyzable policy are unchanged.
This keeps the change separate from nearby PRs #2735, #2748, and #2767.

## Validation

The full locked workspace Rust suite passed with PostgreSQL 18. All 579 compiled
native API tests passed in 21 suites. The CLI regression checks the issue's two
foreign keys at lines 1 and 2, later-only suppressions and allow comments, and
multiline statements with 1, 4, and default workers. Native tests cover CommonJS,
ESM, and prepared project reports. `cargo clippy`, formatting, spelling, duplicate checks,
import resolution, and the configured repository check passed.

Full-suite coverage was 99.32% lines, 99.34% functions, and 98.54% regions.
Changed-line coverage was 288/289 (99.65%); no filenames were excluded. No function
exceeded CRAP 30. Coverage was measured before removing obsolete test-only lexical
helpers and applying two field-name shorthand fixes; runtime behavior is identical.
The 12GB coverage target was removed after evidence export.

Structural impact commands returned no Rust dependents or selected Cargo tests,
so validation used the full configured workspace scope. Signature impact hit the
repository's intentional invalid-UTF-8 fixture; the JS import check passed.

## Reproduce the fixture audit

From the repository root, build distinct baseline and candidate CLIs, then run:

```sh
python3 docs/work-evidence/postgres-statement-lines-2760/compare.py \
  /path/to/baseline/no-mistakes /path/to/candidate/no-mistakes --out /tmp/locations-audit
```

The baseline is `ff448f7b340af306a6fa6496c90dc132449ada1a`. The corpus reuses the
saved issue-2734 configuration manifest plus the 15 additions here. All 1,090
configurations produced identical stdout, stderr, and exit status at default,
1, and 4 workers. Findings/advisories cover all 32 registered Postgres rules.
The 36 existing configuration errors match the baseline and are not counted as
successful analyses.

Of these configurations, 1,083 are byte-identical to the baseline. The seven
changes are corrected statement locations, suppressions applying only to their
own declaration, and repeated findings no longer collapsing onto the same wrong
line. Existing recursive-view fixtures now retain two separate policy findings
instead of one deduplicated finding. See `validation.json` for the affected
configurations, finding counts, and frozen binary hashes.
