# `no-mistakes/forbidden-member-chains`

## Why

Some policies ban a member path no matter which receiver it hangs off, and
whether or not the path is called. Call selectors only see invocations, and an
exact callee spelling changes with every receiver. This rule matches the static
chain itself. Names, receiver wildcards, and messages all come from
configuration. The rule does not hardcode an API and does not read a call graph.

## Disallowed

```ts
records.entry;
records.entry();
other.panel.open;
other.panel['open']();
new other.drawer.toggle();
```

## Allowed

```ts
records.entries;
panel.open;
other.panel.close;
const label = 'other.panel.open';
// other.panel.open
other.panel[method];
<Panel.Open />;
```

## Options

The rule is opt-in. An omitted `chains` list matches nothing. Use ordinary
ESLint flat-config `files` or oxlint overrides for file scope. This rule has no
glob option.

`chains` has at most 64 entries. Each entry requires `path` and `message`.
`path` is a dotted string or an array of segments. A leading `*` segment, or
`receiver: "*"`, means any receiver followed by the remaining segments. A named
`receiver` is the explicit root and cancels a leading `*`. Array segments are
literal property names, so a dot inside one segment is not a separator. At most
16 segments are proven. Longer paths, empty segments, and a `*` that is not the
receiver are ignored.

```js
{
  files: ['src/legacy/**/*.ts'],
  rules: {
    'no-mistakes/forbidden-member-chains': ['error', {
      chains: [
        { path: 'records.entry', message: 'Use the entry helper.' },
        { path: '*.panel.open', message: 'Open the panel through the helper.' },
        {
          receiver: '*',
          path: ['drawer', 'toggle'],
          message: 'Toggle the drawer through the helper.'
        }
      ]
    }]
  }
}
```

The match is the static member chain. Dot access and literal bracket keys are
the same segment. String literals, numeric literals, and no-substitution
template literals count. `this`, `super`, and `import.meta` are static roots.
Private names include `#`. Optional `?.` does not change the chain.

Transparent wrappers that do not change the chain are ignored: parentheses,
`as` assertions (including `as const`), `satisfies`, non-null `!`, angle-bracket
assertions, and type instantiation. A member read reports on the outermost
matching member. A trailing call or `new` whose callee is that member is the
same access and reports once, on the invocation. A repeated suffix inside one
chain reports once. Two configured chains may both report on one access.

Dynamic computed keys are explicitly unproven and do not match. That includes
identifiers, calls, concatenations, and templates with substitutions. A static
tail after a dynamic key can still match a receiver wildcard; the key itself is
not guessed. Strings, comments, and JSX member names are not member chains.
Unrelated paths do not match. The walk proves at most 32 member segments;
deeper chains are unproven.

## Fix

Replace the access with the API named in `message`, or stop reading that chain.
There is no automatic fix. The rule does not execute the member path.

## Suppression

A reason is optional. This rule does not require or validate one.

```ts
// no-mistakes-disable-next-line forbidden-member-chains
records.entry;
records.entry(); // no-mistakes-disable-line forbidden-member-chains
```

`no-mistakes-disable-file` before the first token suppresses the whole file.
`no-mistakes/forbidden-member-chains` is the same rule id. A directive-looking
string does not suppress a finding. ESLint's own suppression comments still
work when the file is linted with ESLint.

## Related rules

- [`forbidden-calls`](../rules/forbidden-calls.md) bans calls through the
  prepared call graph. It does not see a member read that is not a call.
- [`no-banned-import-outside-allowed-paths`](no-banned-import-outside-allowed-paths.md)
  restricts where a module may be imported.
