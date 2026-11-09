# Empty React component proof corpus

Run `python3 materialize.py /tmp/react-empty-components-corpus` once to create a
separate immutable corpus. This does not modify the integration discovery corpus.
The saved seed expands to 256 service modules, each with 64 exported lowercase
functions and callbacks using object spreads, property reads, maps, filters, and
reductions. A shared lowercase helper adds one module. The resulting sources are
about 12 MB; record exact source bytes/lines after materialization.

`configs/default.yml` requests default forbidden-dependency relationships,
including React render edges. Existing component extraction proves every module
has no exported component, so detailed React analysis can be skipped. The
`configs/import-only.yml` control requests no React facts. Both configurations
must report 256 forbidden dependencies on `shared/helper.mts` with status 1.

Measure fresh matched invocations at jobs 1, 4, and default, with diagnostics off
and on, using the same balanced blocks and resource/report parity checks as the
integration discovery driver. Keep this corpus separate, verify binary source
provenance, and compare complete normalized reports and exit statuses. The saved
fixtures and work counters prove the negative path; no timing improvement is
claimed until the matched consuming and control measurements pass.

For a positive component control, also run the unchanged integration-discovery
materializer into a third fresh corpus, copy this directory's
`configs/component-control.yml` into that corpus, and record its separate
immutable hash. This config uses default relationship selection and an absent
forbidden path, so it reports zero findings with status 0 while exercising
files that do contain React components. Keep this corpus separate from both
the integration-discovery and empty-component corpora.
