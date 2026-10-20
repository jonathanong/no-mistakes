<!-- cspell:ignore mimalloc codegen -->

# Scoped allocator observations

This evidence is scoped and does not approve allocator adoption. The PR remains
[draft](https://github.com/jonathanong/no-mistakes/pull/2200).

[Tables](tables.md) report wall time, CPU and peak RSS for the reviewed 131-rule
consumer cohort at 1/4/max jobs and a completing annotation fixture at 1/max.
Diagnostics off/on are separate, with four matched pairs per cell. All 20 cells
and 160 native processes completed. The annotation fixture is synthetic; its
parallel RSS observations are higher under mimalloc on both platforms.

Measured native source: `8c4ca1d58a6d0322780333c743a2cf98f5e3b80f`.
Linux uses fat LTO/one codegen unit and automatic four workers; macOS uses thin
LTO/16 codegen units and automatic three workers. Explicit four-worker macOS
rows exceed its automatic count. Comparisons stay within each platform/host.

The [original run](https://github.com/jonathanong/no-mistakes/actions/runs/39467428353)
built all four native artifacts and completed macOS. Its Linux dependency setup
failed before sampling. The [Linux retry](https://github.com/jonathanong/no-mistakes/actions/runs/39474778706)
reused those immutable artifacts and checked out the same measured source;
workflow-only `04b8501eb` fixed isolated Python setup. No failed sample is pooled.

- [Samples](samples.json) retain every raw native resource report, report hashes,
  exact argv, worker and current-launcher RSS metadata.
- [Summary](summary.json) retains all resource samples, medians, maxima and paired
  ratios. An independent raw-row recomputation matches every reported value.
- [Artifact origins](artifact-origins.json) retain API identities and ZIP digests.
- [Build proofs](build-proofs.json) retain native compiler/profile/hash provenance
  and actual Linux addon dynamic/TLS evidence.
- [Checksums](checksums.json) bind these retained evidence files.

Complete JSON/status/semantic-stderr parity and source/config/inventory identities
were verified before archiving. Original stdout/stderr and native binaries remain
in the linked run artifacts under their retention policy. These scoped cells
cannot replace full-config, N-API, many-core or broader release adoption gates.
