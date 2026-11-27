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
  "constructors": ["Intl.DateTimeFormat", "Intl.NumberFormat"],
  "allowUndefinedLocale": false,
  "unknownValues": "ignore"
}
```

- `methods` replaces the default method-name list with non-blank method names.
  An empty list disables method checks. Selecting `toLocaleString` also checks
  Number and custom-object methods with that name.
- `constructors` defaults to an empty list. Each entry selects a member of a
  global namespace by its dotted `Namespace.Member` name and checks both calls
  and `new` expressions. A shadowing local object stays quiet. When the dotted
  name is listed and `unknownValues` is `"finding"`, the rule reports
  `unanalyzable` for a missing locale when the namespace comes from an import,
  a global `require()` call or member access, or `import = require()`. Local
  functions named `require` and other local initializers are outside this
  check. A direct assignment of a local object before the call in the same
  statement list also stays quiet.
  An `Intl.*Format` constructor or call whose only use is an immediate
  `.resolvedOptions()` call (including any property read on its result, such as
  `Intl.DateTimeFormat().resolvedOptions().timeZone` or
  `new Intl.NumberFormat().resolvedOptions().locale`) is exempt; storing
  the instance in a variable or passing it elsewhere is still reported.
- `allowUndefinedLocale` permits global `undefined` or a `void` expression as
  the explicit first argument. Omitting the locale still produces a finding.
- `unknownValues` defaults to `"ignore"`. Set it to `"finding"` to report
  undecidable calls on configured methods or constructor namespaces: spread
  first arguments (`date.toLocaleString(...args)`,
  `new Intl.DateTimeFormat(...args)`), computed constructor lookups on a
  configured namespace (`new Intl[name]()`, including through a stable
  constructor alias), `undefined` or matched alias and constructor
  references inside dynamic `with` or non-strict direct `eval` scopes, and
  a configured constructor whose namespace is an import, `require()`, or
  `import = require()` this file cannot read. That last form reports
  `unanalyzable`, not a missing-locale finding.

Invalid options—such as blank `methods` entries, non-dotted `constructors`
entries, or unknown `unknownValues` strings—fail schema validation at config
load.

With the options above, both of these expressions report a missing locale.

```ts
new Intl.DateTimeFormat();
Intl.NumberFormat(undefined);
```

Pass the application's locale to fix the findings.

```ts
new Intl.DateTimeFormat("en-GB", { timeZone: "UTC" });
Intl.NumberFormat("en-GB");
```

A proven local object stays quiet:

```ts
const Intl = { DateTimeFormat() { return { format() {} }; } };
new Intl.DateTimeFormat();
```

An `import Local = Other` alias of a name in this file stays quiet. An
unread module reports `unanalyzable` once `unknownValues` is `"finding"`
and `constructors` lists that dotted name:

```ts
import * as Intl from "@example/intl";
import IntlEquals = require("@example/intl");
const IntlRequired = require("@example/intl");
const IntlMember = require("@example/intl").Intl;

new Intl.DateTimeFormat();
new IntlEquals.DateTimeFormat();
new IntlRequired.DateTimeFormat();
new IntlMember.DateTimeFormat();
```

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
helper name matches a configured method. Optional calls, static string and
expression-free template member keys, and same-file `const` aliases of those
keys are checked. Constructor entries match direct references such as
`Intl.DateTimeFormat` and same-file `const` aliases of those references.
TypeScript assertions on the aliases are transparent. Dynamic method keys,
qualified namespaces such as `globalThis.Intl`, detached instance methods, and
indirect `.call`/`.apply` invocations are outside the rule.
A spread first argument, a dynamic computed member on a configured constructor
namespace, and references in dynamic `with` or non-strict direct `eval` scopes
stay unknown under the default `unknownValues: "ignore"` and report findings
when `unknownValues: "finding"` is enabled. Aliases and other dynamic locale
expressions are accepted; the rule does not prove their runtime value or validate
locale tags. A binding named `undefined` is resolved in its lexical scope rather
than assumed to be the global value. No TypeScript program is required.

## Fix

Pass the locale from application configuration or the request's negotiated
locale. Use an explicit time zone for date rendering when needed. There is no
automatic fix because the rule cannot choose the correct locale for the app.

## Suppression

Use `no-mistakes-disable-next-line`, `no-mistakes-disable-line`, or a leading
`no-mistakes-disable-file` with either the short rule name or
`no-mistakes/no-locale-less-date-format` (standard `eslint-disable-*`
directives also work):

```ts
// no-mistakes-disable-next-line no-locale-less-date-format -- intentional platform-native display
new Intl.DateTimeFormat();
```

## Related rules

- [`restricted-api-usage`](restricted-api-usage.md) constrains project-specific
  API usage and helper boundaries.
