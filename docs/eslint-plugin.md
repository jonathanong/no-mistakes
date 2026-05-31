# ESLint And Oxlint Plugin

The rule reference now lives in [docs/eslint-rules/](eslint-rules/README.md).

Install:

```sh
npm install --save-dev eslint-plugin-no-mistakes
```

ESLint flat config:

```js
const noMistakes = require("eslint-plugin-no-mistakes");

module.exports = [
  {
    plugins: { "no-mistakes": noMistakes },
    rules: noMistakes.configs.strict.rules,
  },
];
```

Oxlint:

```json
{
  "jsPlugins": ["eslint-plugin-no-mistakes"],
  "rules": {
    "no-mistakes/playwright-literals": "error"
  }
}
```

See [ESLint rule index](eslint-rules/README.md) for per-rule behavior and
options.
