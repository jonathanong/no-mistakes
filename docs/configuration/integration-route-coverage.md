# Integration route coverage

`tests.playwright.routeCoverageSources` lets registered Vitest integration tests
cover a finite explicit set of Next.js page route identifiers alongside Playwright tests.
The default is an empty list. This adds route coverage only: it does not credit
selectors or frontend fetches, or assign integration files to a Playwright project.

```yaml
tests:
  vitest:
    configs: vitest.config.ts
  playwright:
    routeCoverageSources:
      - framework: vitest
        project: web-integration
        include: [integration/web/**/*.mts]
        routes: [/health, /legal/copyright]
        helpers:
          - module: integration/web/helpers/client.mts
            export: WebIntegrationClient
            method: request
            urlArgument: 0
          - module: integration/web/helpers/client.mts
            export: WebIntegrationClient
            method: loadPage
            urlArgument: 0
```

An app binding at `tests.playwright.apps.<playwright-project>.routeCoverageSources`
can replace the top-level list. Its routes belong to that bound frontend app.

Every source requires `framework`, `project`, `include`, `routes`, and `helpers`.
Currently `framework` is `vitest`. `project` must identify a project in the
effective Vitest runner config. `include` selects candidate registration modules,
including modules statically imported by runner-owned test entries. A matching
file without a registered entry owner receives no credit. Type-only and dynamic
imports do not establish registration ownership.

`routes` contains exact canonical page route identifiers in the bound app,
including named parameter routes such as `/user/:id/posts/saved`. Wildcard
exemptions, query strings, fragments, and undeclared app routes are errors.
Literal and template request paths use normal route matching and specificity;
only the most specific actual route may receive credit, and its canonical
identifier must appear in this finite list. A wrong suffix cannot cover a
declared route, and a more specific literal page cannot credit a parameter route.

Each helper identifies an exact repository-relative module file or resolvable
module specifier and an exported binding. Import aliases are supported. Set
`method` for an imported class instance method; omit it for an imported function.
`urlArgument` is the zero-based argument containing the route string. Class
receivers must have a concrete imported constructor. Shadowed, reassigned, or
ambiguous bindings receive no credit.

Only statically bound `test`/`it` callbacks from `vitest`, optionally nested in
bound `describe` callbacks, contribute occurrences. Skipped and conditional
registrations, unbound declarations, uncalled function bodies, and wholly dynamic
URLs receive no credit. Template interpolation can match named route parameters.
HTTP status assertions do not affect attribution.

Route `tests` names the registered runner entry file. `testsDetail[].attribution`
and route edges preserve `framework`, `project`, and `declarationFile`, so a
registration module remains distinguishable from the runner entry that executes
it. The canonical `route-test` edge connects that runner entry to the route, allowing Vitest
impact planning to retain the correct framework.

The same configured behavior is available through the asynchronous
`playwrightCheck`, `playwrightEdges`, and `resolveConfig` Node APIs.
