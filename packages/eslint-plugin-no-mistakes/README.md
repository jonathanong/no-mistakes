# eslint-plugin-no-mistakes

ESLint and Oxlint rules that keep TS/JS, React, Next.js, and Playwright code
static enough for `no-mistakes` analyzers. They exist so agents cannot hide
callers, fetches, or selectors behind aliases and dynamic values. See
[why no-mistakes exists](../../docs/why.md).

```sh
npm install --save-dev eslint-plugin-no-mistakes
```

See the [ESLint rule index](../../docs/eslint-rules/README.md).
