# SQL verdict resource evidence

This generated snapshot preserves [ARM run 39488578982](https://github.com/jonathanong/no-mistakes/actions/runs/39488578982). The measured candidate is `bcbbf58d69f8157aab60e5f1e61decf68ad278b5`; frozen main is `4d4deb9423111f6bd97f2501bdf508ef17536dc0`; harness is `510a88d5102b230b4fb626051c4141f670aedf10`.

[Both resource tables](tables.md) report the ordering-only cohort at 1, 4 and maximum jobs (maximum is 4 on this ARM runner), diagnostics off/on and ledger off/on. Each cell has three same-VM main/main control blocks and three balanced baseline/candidate/candidate/baseline blocks, six adjacent pairs per stage. All 288 arms finished uncensored.

All six ledger-off wall deltas (+0.006% to +1.149%) are within their observed same-VM control spread. This finite sample comparison is descriptive, not a confidence bound. The maximum-jobs, diagnostics-off control spread is 7.607%. Ledger-on wall deltas are +0.462% to +2.848%; two maximum-jobs cells exceed their own control spread. No flat ledger-on, full-configuration, or newer-main claim is made. The PR remains draft.

The twelve original `raw/samples.json` files contain all 288 resource records, including CPU timelines and stdout/stderr hashes. Original artifact metadata, ZIP digests, per-file checksums, machine/build/workload provenance, the pre-data analysis plan, source identities, validation results, and independent recomputation are retained. Other JSON files are generated evidence, not handwritten source.

Run `python3 recompute.py` from any directory to verify every preserved file hash, recompute role means and adjacent pair deltas/spreads, and compare all decisions with the independently checked summary. This standalone verifier reads the saved resource records only. It does not rebuild or run the analyzer.

Full stdout reports, raw stderr streams, binaries and duplicate block resource arrays are omitted. `summary-projection.json` records the original summary hash and the 36 duplicate complete ledger dictionaries removed from its compact preserved projection; all numerical fields, decisions, ledger hashes and validation flags are unchanged. Their original hashes remain in the records and artifact manifests. Full output/exit parity, source/compiler/flags/binary identity, physical workload and all 2,155 ledger obligations were verified before preservation; the standalone resource verifier cannot reconstruct omitted report bytes. The original collector and harness are pinned by their hashes in `analysis-plan.json`.

The evidence-only follow-up does not change analyzer source or dependencies. Full correctness and 98% coverage gates apply to the measured candidate; no new runtime measurement is attributed to the evidence commit.
