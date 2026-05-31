# `no-mistakes playwright related`

Print Playwright tests that cover route or component files.

```sh
no-mistakes playwright related web/app/users/page.tsx --json
```

Use this for targeted browser test selection after changing a page, route, or
component with tracked selectors.

Shared Playwright options are documented in [`playwright`](playwright.md).

Node API: `playwrightRelated(options)`.
