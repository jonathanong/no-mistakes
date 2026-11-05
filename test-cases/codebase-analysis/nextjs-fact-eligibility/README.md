# Next.js fact eligibility (#3029)

`fixture/minimal` reproduces the issue with only `index.ts` and `data.json`.
Configuration stays outside the source directory. `combined.yml` must exit 0
without warnings, as must both controls: removing dependency declarations
(`cache-only.yml`) and explicitly including TS/JS (`include.yml`).
`combined-low.yml` repeats the regression at low effort.

`fixture/mixed` keeps one caching violation and one real API route. JSON and
MDX under both Pages Router prefixes must never become caching inputs or API
route findings. `mixed.yml` and `mixed-low.yml` combine dependency and Storybook
fact demand; `storybook.yml` widens the plan through MDX without dependency
imports. Each reports exactly the two source violations without warnings.
`api-only.yml` and `api-dependencies.yml` report the same single source route.

The CLI assertion test verifies exit codes, warnings, and source-only findings.
The committed JSON snapshot compares jobs 1 and 2 and records exact diagnostics.
