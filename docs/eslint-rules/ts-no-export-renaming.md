# `no-mistakes/ts-no-export-renaming`

Disallows value export renaming.

Why: renamed exports make symbol-level dependency tracing less direct.

Counterexample: `export { handler as GET }`.

Fix: export values under their declaration names, or rename the declaration
itself.
