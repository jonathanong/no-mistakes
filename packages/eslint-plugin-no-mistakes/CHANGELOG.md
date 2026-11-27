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
