# `no-mistakes playwright`

Analyze Playwright route, selector, fetch, and assertion coverage.

| Leaf command | Purpose |
| --- | --- |
| [`playwright check`](playwright-check.md) | Fail on uncovered configured routes/selectors or duplicate selectors. |
| [`playwright edges`](playwright-edges.md) | Print test-to-route and test-to-selector edges. |
| [`playwright related`](playwright-related.md) | Find tests related to route/component files. |
| [`playwright tests`](playwright-tests.md) | Print assertions grouped by test. |

Shared options: `--root`, `--config`, repeatable `--playwright-config`,
`--project`, `--json`, `--assert-conditional-tests`,
`--allow-skipped-tests`, `--assert-unique-test-ids`,
and `--assert-unique-html-ids`.

`--project <NAME>` selects a top-level Playwright config name (or, for a
single unnamed config, a `projects[].name` inside it) — it does not select a
frontend app. When the repository configures more than one `type: nextjs`
project, set
[`tests.playwright.apps.<project>.project`](../configuration/tests.md#multiple-frontend-apps)
so these commands know which app `--project` exercises; without it, an
ambiguous repository fails with an error naming the candidate apps instead of
guessing.
