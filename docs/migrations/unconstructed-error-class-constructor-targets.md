# Unconstructed Error classes: unresolved targets are advisory

The [#2734 section 2](https://github.com/jonathanong/no-mistakes/issues/2734)
release fix corrects false failures introduced by #2699 in
[`unconstructed-error-class`](../rules/unconstructed-error-class.md). The rule
remains opt-in and has no new configuration fields.

Static constructor targets reuse the prepared resolver, including tsconfig path
aliases, re-exports, and usable package class declarations. A resolved class is
credited as constructed even when its own ancestry is unresolved. Known non-Error
globals such as `Intl.DateTimeFormat` and `Intl.NumberFormat` no longer produce
unresolved-target diagnostics. Shadowed global names need independent proof.

Still-unresolvable constructors, their otherwise-unused enclosing classes, and
otherwise-unused classes whose bases remain unresolved now produce non-failing
advisories, including when `unanalyzable: report` is selected explicitly or
inherited from the repository. This includes unavailable package or host-module
declarations; an unresolved base alone does not prove that a class extends Error.

`unanalyzable: advise` also advises. `unanalyzable: ignore` omits these
diagnostics. Namespace-escape-only uncertainty keeps the shared policy: `report`
fails, `advise` advises, and `ignore` omits it. Definite unused Error classes
still fail under every policy.

CLI `check`, Node `check()`, and check reports in `analyzeProject()` list these
advisories in `advisories`, separate from failing `rules`. The standalone Rust
findings-only API omits advisories. Ordinary import and call graph query results
retain their existing meaning.

Prefer named constructors and analyzable class declarations to remove the
uncertainty. For intentional dynamic construction, line, next-line, and file
suppression directives still apply. Suppress the construction and its class
declaration separately when both have diagnostics:

```ts
// no-mistakes-disable-next-line unconstructed-error-class: constructed by the runtime
export class RuntimeError extends Error {
  static create() {
    // no-mistakes-disable-next-line unconstructed-error-class: runtime chooses the subclass
    return new this();
  }
}
```

`no-mistakes-disable-line unconstructed-error-class` works on the reported line;
`no-mistakes-disable-file unconstructed-error-class` in the leading comment
block suppresses the whole file. Suppressions apply to findings and advisories.
