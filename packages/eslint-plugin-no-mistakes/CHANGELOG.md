# Changelog

## Unreleased

### Breaking changes

The recommended ESLint configuration now errors on type re-exports that
`no-placeholder-never-type-exports` cannot verify from the current file.
Use an explicit rule suppression for verified re-exports that must stay.
