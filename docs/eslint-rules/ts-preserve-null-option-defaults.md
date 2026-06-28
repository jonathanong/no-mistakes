# `no-mistakes/ts-preserve-null-option-defaults`

Disallows defaults that collapse explicitly nullable option members.

Why: when an option type says `null` is meaningful, defaults must distinguish
an omitted value from an explicit `null`.

```ts
interface Options {
  label?: string | null;
}

function render(options: Options) {
  return options.label ?? "Untitled";
}
```

Counterexample: a nullable optional member is defaulted with `??`, `||`, `??=`,
or `||=`.

Fix: check for `undefined` explicitly and preserve `null`.

```ts
function render(options: Options) {
  return options.label === undefined ? "Untitled" : options.label;
}
```

Options: `includePathPatterns`, `excludePathPatterns`, `optionObjectNames`, and
`optionObjectNamePatterns`.

Object destructuring defaults are allowed because they only apply when the
property value is `undefined`, preserving explicit `null`.
