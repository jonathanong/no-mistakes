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

The [#2742 follow-up](https://github.com/jonathanong/no-mistakes/issues/2742)
resolves constructor facades and immutable aliases through the same session's
prepared binding facts, canonical graph, and declaration resolver projection.
Each importer keeps its own tsconfig scope. Workspace package root and wildcard
subpath exports follow the canonical workspace resolver through star and named
re-exports, including definitions outside the rule include. Resolved unrelated
star exports are skipped; unresolved or conflicting exports remain uncertain.
Package members declared as
`typeof NamedClass` can supply concrete class identity, including a base chain
that proves a class is not an Error. A construct signature alone does not prove
that ancestry. Included ambient module declarations also supply base-class proof
within the importing project.

The [#2753 follow-up](https://github.com/jonathanong/no-mistakes/issues/2753)
resolves ambient modules that a project's tsconfig `types` list makes visible,
including relative declaration files and packages, and values inherited from a
relative extended config. A `declare module` that uses `export = Namespace`
supplies that namespace's exported classes. A declared non-Error base is silent.
A declared Error base still reports. A specifier with no visible declaration
stays advisory. No module name is special-cased. Unset `types` does not scan
installed packages, and this proof does not add those files to ordinary import
or call query results. Nullish constructor choices narrow only when the selected class
value is provably non-nullish; unknown alternatives remain uncertain.

Declaration proof does not add package files to ordinary import/call query
results. Genuine unused Error subclasses still report, and a construction
credits its exact class rather than another class with the same name.

Still-unresolvable constructors, their otherwise-unused enclosing classes, and
otherwise-unused classes whose statically named bases remain unresolved now
produce non-failing advisories, including when `unanalyzable: report` is
selected explicitly or inherited from the repository. This includes unavailable
package or host-module declarations; an unresolved base alone does not prove
that a class extends Error. Dynamic base ancestry follows the shared policy
and fails under `report`.

`unanalyzable: advise` also advises. `unanalyzable: ignore` omits these
diagnostics. No option turns an unresolved constructor or a statically named
unresolved base into a failure. `report` and `advise` differ for dynamic base
ancestry and when namespace escape is the class's only uncertainty.
Namespace-escape-only uncertainty keeps the shared policy: `report` fails,
`advise` advises, and `ignore` omits it. Definite unused Error classes still
fail under every policy.

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
