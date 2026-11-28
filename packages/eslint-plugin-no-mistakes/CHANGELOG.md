# Changelog

## Unreleased

### Breaking changes

The recommended ESLint configuration now errors on type re-exports that
`no-placeholder-never-type-exports` cannot verify from the current file.
Use an explicit rule suppression for verified re-exports that must stay.
For a file of verified re-exports, add
`// no-mistakes-disable-file no-placeholder-never-type-exports -- verified re-exports`
at the top of the file. A flat-config override can also disable this rule
for the corresponding file globs.

`react-no-use-promise-resolve`, `react-no-nullish-react-node`,
`nextjs-no-manual-script-tags`, and `server-require-nullable-fetch-wrapper`
now report `unanalyzable` when a binding cannot be traced. Set
`unanalyzable: "ignore"` on that rule to restore the previous silence.
