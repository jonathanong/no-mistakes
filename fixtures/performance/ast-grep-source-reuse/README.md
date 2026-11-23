# Ast-grep source reuse fixtures

The `single`, `repeated-12`, and `repeated-20` roots configure one, twelve,
and twenty named applications of `ast-grep-ignore-freshness`. Each application
selects the same ten saved rule YAML files with `files: [rules]` and
`include: [rules/*.yml]`. Two saved source files make every ignore live.
All three reports have zero findings, warnings, and advisories, isolating YAML
reuse from suppression filtering. The root config is not a selected rule YAML.

The separate `semantics` root tests merge keys, partial-document recovery,
and strict invalid-UTF-8 errors; it is not timed. See
[measurement evidence](../../../docs/performance/shared-source-suppression.md)
for the release-build methodology and results.
