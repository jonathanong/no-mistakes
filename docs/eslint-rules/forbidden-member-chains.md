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
const { open } = other.panel;
const { open: show } = other.panel;
const { panel: { open } } = other;
function f(records) {
  return records.entry;
}
const method = "open";
other.panel[method];
declare const unbound: string;
other.panel[unbound];
const { [unbound]: opened } = other.panel;
```

## Allowed

```tsx
records.entries;
panel.open;
other.panel.close;
const label = "other.panel.open";
// comment mentioning other.panel.open is not a member access
const { close } = other.panel;
const { open } = panel;
const { ["open"]: show } = other.panel;
function f(records) {
  return records.entries;
}
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
16 segments are allowed. A named receiver counts as one, and it replaces a
leading `*` instead of adding another segment. A leading `*` that stays a
wildcard still counts toward the 16, so `*.` plus 15 segments is the longest
path of that shape. `receiver: "*"` does not add a segment and may be followed
by 16. The schema rejects an empty segment (`records..entry`), a `*` that is
not the receiver (`records.*.entry`, a lone `*`, or a `*` later in an array),
and a longer path. ESLint and oxlint report a configuration error instead of
dropping the chain. A sibling chain in the same list does not run.

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
the same segment. String literals, numeric literals, boolean literals, `null`,
and no-substitution template literals count. A unary `+` or `-` on a numeric
literal is that number. A `const` binding whose initializer is one of those is
the same segment, one binding deep. A later alias is not followed. `let`,
`var`, parameters, and imports are not followed. `this`, `super`, and
`import.meta` are static roots. Private names include `#`. Optional `?.` does
not change the chain. A bigint key is not a string segment, so `matrix[1n]`
does not match `matrix.1`.

Transparent wrappers that do not change the chain are ignored: parentheses,
`as` assertions (including `as const`), `satisfies`, non-null `!`, angle-bracket
assertions, and type instantiation. A member read reports on the outermost
matching member. A trailing call or `new` whose callee is that member is the
same access and reports once, on the invocation. A repeated suffix inside one
chain reports once. Two configured chains may both report on one access.

A computed key the rule cannot read is unanalyzable when that missing segment
could complete a configured path. The finding names the path and repeats the
configured message. A key the rule can read that is a different segment stays
quiet, including `true`, `-1`, and `records.entries` against `records.entry`.
A static tail after an unknown key still matches a receiver wildcard when the
known suffix is the path (`obj[name].panel.open`). The unknown key is not
guessed as a definite segment. An unknown key cannot be a private `#` name, so
it does not match `*.#slot`. Strings, comments, and JSX member names are not
member chains. The walk proves at most 32 member segments. A deeper chain is
unproven and stays quiet.

Object destructuring is the same access. `const { open } = other.panel`, a
rename `const { open: show } = other.panel`, and a nested pattern
`const { panel: { open } } = other` match `*.panel.open`. So does a default on
a parameter, `function f({ open } = other.panel) {}`, and a default at any
nesting depth, `const { x: { open } = other.panel } = {}`. An object pattern
in an array element is the same kind of default,
`const [{ open } = other.panel] = []`. The finding is the property whose key
completes the chain. A repeated suffix inside one pattern reports once. The
outer pattern and its default share that pair, so
`const { panel: { open } = other.panel } = other` reports `*.panel.open` once.
An identifier computed pattern key follows the same `const` rule.
`const { [method]: open } = other.panel` matches when `method` is a `const`
string or number. An identifier the rule cannot read, such as
`const { [unbound]: open } = other.panel`, is unanalyzable when it could
complete a configured path. Other computed pattern keys stay unproven and are
not reported, including a literal computed key such as `{ ["open"]: show }`.
A non-computed key may be an identifier or a string or numeric literal. A
static tail on the receiver can still match a wildcard after a dynamic member
(`const { open } = other[name].panel`).

Named roots are file-local spellings, not resolved bindings. A parameter or
local does not hide a chain. `function f(records) { return records.entry }` is
reported for `records.entry`. Scope is read only for one `const` initializer
behind a computed key.

## Fix

Replace the access with the API named in `message`, or stop reading that chain.
That includes an object destructure of the banned segment. There is no
automatic fix. The rule does not execute the member path.

## Suppression

A reason is optional. This rule does not require or validate one.

```ts
// no-mistakes-disable-next-line forbidden-member-chains
records.entry;
records.entry(); // no-mistakes-disable-line forbidden-member-chains
```

The same comments remove an unanalyzable finding. An unsuppressed copy of that
access still reports. `no-mistakes-disable-file` before the first token
suppresses the whole file.
`no-mistakes/forbidden-member-chains` is the same rule id. A directive-looking
string does not suppress a finding. ESLint's own suppression comments still
work when the file is linted with ESLint.

## Related rules

- [`restricted-api-usage`](restricted-api-usage.md) has opt-in lexical selectors
  for bare references, declarations, literal object keys, and constructor arity
  when only a name, not a member path, is forbidden.
- [`forbidden-calls`](../rules/forbidden-calls.md) bans calls through the
  prepared call graph. It does not see a member read that is not a call.
- [`no-banned-import-outside-allowed-paths`](no-banned-import-outside-allowed-paths.md)
  restricts where a module may be imported.
