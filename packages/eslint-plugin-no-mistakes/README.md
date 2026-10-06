# eslint-plugin-no-mistakes

ESLint and Oxlint rules that keep TS/JS, React, Next.js, and Playwright code
static enough for `no-mistakes` analyzers. They exist so agents cannot hide
callers, fetches, or selectors behind aliases and dynamic values. See
[why no-mistakes exists](../../docs/why.md).

```sh
npm install --save-dev eslint-plugin-no-mistakes
```

Configure the plugin in a flat config and scope rules with `files` globs. For
example, this checks test files for skipped or exclusive tests while allowing
`todo`:

```js
import noMistakes from "eslint-plugin-no-mistakes";

export default [{
  files: ["**/*.test.*", "tests/**"],
  plugins: { "no-mistakes": noMistakes },
  rules: { "no-mistakes/test-no-skips": ["error", { allow: ["todo"] }] },
}];
```

See [`test-no-skips`](../../docs/eslint-rules/test-no-skips.md) for the rule's
behavior and configuration, and the [ESLint rule index](../../docs/eslint-rules/README.md)
for the complete list.
