# `no-mistakes/no-placeholder-never-type-exports`

Disallows exported `never` placeholder type aliases.

Why: placeholder exports look like public API but carry no usable contract.

Counterexample: `export type User = never`.

Fix: replace the placeholder with the real type or remove the export.
