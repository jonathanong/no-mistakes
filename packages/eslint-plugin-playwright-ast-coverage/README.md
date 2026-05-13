# eslint-plugin-playwright-ast-coverage

ESLint and Oxlint rules for keeping Playwright test IDs static, defaulted, and
consistent with `playwright-ast-coverage`.

## ESLint

```js
const playwrightAstCoverage = require("eslint-plugin-playwright-ast-coverage");

module.exports = [
  {
    files: ["**/*.{js,jsx,ts,tsx}"],
    plugins: { "playwright-ast-coverage": playwrightAstCoverage },
    rules: playwrightAstCoverage.configs.strict.rules,
  },
];
```

## Oxlint

```jsonc
{
  "jsPlugins": ["eslint-plugin-playwright-ast-coverage"],
  "rules": {
    "playwright-ast-coverage/literals": "error",
    "playwright-ast-coverage/defaults": "error",
    "playwright-ast-coverage/unique": "error"
  }
}
```

`configs.recommended` enables `literals`, `defaults`, `no-empty`, and `unique`.
`configs.strict` also enables canonical attribute, naming, interactive element,
and `getByTestId` preference rules.
