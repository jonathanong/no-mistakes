# Analysis scope and shared facts

Each `analyzeProject()` report may use its report-specific options. Graph
reports may override `root`, `tsconfig`, and `config`; `resolveCheckDependencies`
accepts the same scope overrides, and referenced dependency report IDs are
resolved within that report's effective scope. `reactUsages` accepts
`target`, `targets`, `include`, and scope options; and `check` may override
`root`, `tsconfig`, and `config`. Lightweight queries (`importers`, `exportsOf`,
`deadExports`, `callSites`, `resolveCheck`), `fetches`, test-plan reports,
`lockfileDiff`, CI/infra/swift reports, `impactedChecks`, and
`validateMermaidMarkdown` are also valid `reports[].type` values. They inherit
the request `root`/`tsconfig`/`config` and dispatch through the dedicated Node
APIs. `callSites` shares the request's prepared argument and call facts,
including namespace calls and static lexical aliases while excluding shadowed
bindings. Its ordinary resolver scope matches the standalone query even when
companion reports use broader runner catalogs or ignored explicit roots;
equivalent call graphs are reused. A `symbols` list report paired with `check`
retains the standalone symbols result for every explicit file, including a
visible file outside configured check candidates and a file with no exports.
`effects()` retains supplier read timing for synchronous, immediately invoked
returned helpers imported directly or through re-exports. Later receiver writes
do not erase an earlier proven call; prior writes, escaping helpers, and
conflicting inputs remain conservative. CLI and async CJS/ESM APIs share this
prepared analysis.

`effects` also supports binding-aware
`effects.<kind>.targets` config
selectors by exact import specifier and export path; its string lists retain
spelling-based matching. The `query-reached-per-item` rule in `check()` and
`analyzeProject()` check reports honors those same binding-aware selectors,
including transitive helper calls, factory returns, and agreeing client
argument/default bindings, including qualified named-client command lists,
without matching unrelated same-named bindings.
`effects.<kind>.returnedExecutors` selects callable results of imported factories
whose implementation is opaque; both APIs honor these selectors.
`effects.<kind>.typedExecutors` selects calls through parameters with explicit
callable types imported from the configured module/export, including aliases,
namespace imports, explicit type re-exports, and static indexed property types
wrapped in `NonNullable`. Both APIs also preserve returned-executor metadata in
statically invoked async callbacks and select nullish choices when every alternative
matches the selected effect family/category. `effects.<kind>.returnedMembers`
selects exact member calls on receivers returned by configured imported factory
paths. Both selectors share prepared call provenance across asynchronous
`effects()`, `check()`, and `analyzeProject()` reports; categories and selector
paths are reflected in effects query results. Returned-member callees display
as the factory path plus `().member` (for example,
`writePool.connect().query`); typed executor callees display the configured type
export. These are configuration additions to existing exports and introduce no
new Node functions or option types.
Namespace-local classes, interfaces, type aliases, enums, and type parameters
shadow imported types, including declarations later in the same lexical body.
A value-only local binding does not shadow a type import. Leaving that scope
restores the imported annotation; matching still requires the configured module
and export identity.
`resolveCheck` import rows include `computed: true` for non-literal
`import()` / `require()` specifiers, which are classified `unresolved`.
`resolveCheck` also resolves recognized workspace package imports through the
same visible `exports`/`main` resolver as dependency `workspace` edges; missing or
blocked workspace subpaths are `unresolved`. Third-party and unmatched packages
remain `external`. The same classification applies to `resolveCheckDependencies`
without rediscovering files or parsing its prepared closure again. Configured
aliases keep precedence, and declaration files satisfy only type imports.
`importUsages` omits those computed rows and keeps string literals, including
expression-free templates such as ``require(`./mod`)``.
Reports with the same effective scope share
one request-scoped in-memory dataset. Sources, parsed metadata, and compact file
facts are reused; each normalized graph or symbol-index plan is built at most
once for its file universe. Distinct effective scopes are prepared independently.

When a request omits `tsconfig`, TypeScript/JavaScript imports are resolved with
the config that owns each importing file. Dependency graph and query APIs plus
test planning use this behavior across referenced workspace projects. Set
`tsconfig` only to force that one config for the entire request; this preserves
the previous single-config behavior for debugging and compatibility.

[Node/N-API guide](../node-api.md)
