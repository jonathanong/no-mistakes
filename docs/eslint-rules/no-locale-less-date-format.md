# `no-mistakes/no-locale-less-date-format`

## Why

Implicit locale selection can render different text on the server and in the
browser, causing hydration mismatches. Choose the application's locale
explicitly. For dates, choose a consistent `timeZone` as well when the server
and browser must render identical text; this rule checks the locale argument,
not time-zone options.

## Disallowed

```ts
const date = new Date();
date.toLocaleString();
date.toLocaleDateString(undefined, { timeZone: "UTC" });
date.toLocaleTimeString(void 0);
```

## Allowed

```ts
const date = new Date();
date.toLocaleString("en-GB", { timeZone: "UTC" });
date.toLocaleDateString(["en-GB", "en"]);
formatDate(date);
```

## Options

```json
{
  "methods": ["toLocaleString", "toLocaleDateString", "toLocaleTimeString"],
  "allowUndefinedLocale": false
}
```

- `methods` replaces the default method-name list. An empty list disables the
  rule. Matching uses member names, without receiver type inference: selecting
  `toLocaleString` also checks Number and custom-object methods with that name.
- `allowUndefinedLocale` permits global `undefined` or a `void` expression as
  the explicit first argument. It never permits an omitted locale argument.

## Configuration

The rule is opt-in. Scope it with ESLint flat-config `files` and `ignores`:

```js
import noMistakes from "eslint-plugin-no-mistakes";

export default [{
  files: ["src/**/*.{js,jsx,ts,tsx}"],
  ignores: ["src/formatters/**"],
  plugins: { "no-mistakes": noMistakes },
  rules: { "no-mistakes/no-locale-less-date-format": "error" },
}];
```

## Limits

Direct helper calls such as `formatDate(date)` are never checked, even if the
helper name matches a configured method. Optional calls and static string or
expression-free template member keys are checked. Dynamic member keys, detached
method aliases, and indirect `.call`/`.apply` invocations are outside the rule.
A spread first argument is not expanded. Aliases and other dynamic locale
expressions are accepted; the rule does not prove their runtime value or validate
locale tags. References in dynamic `with` or direct-eval scopes stay unknown.
A binding named `undefined` is resolved in its lexical scope rather than assumed
to be the global value. No TypeScript program is required.

## Fix

Pass the locale from application configuration or the request's negotiated
locale. Use an explicit time zone for date rendering when needed. There is no
automatic fix because the rule cannot choose the correct locale for the app.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/no-locale-less-date-format -- intentional platform-native display
new Date().toLocaleString();
```

## Related rules

- [`restricted-api-usage`](restricted-api-usage.md) constrains project-specific
  API usage and helper boundaries.
