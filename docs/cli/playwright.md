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
`--assert-unique-html-ids`, and deprecated `--assert-unique-selectors`.
