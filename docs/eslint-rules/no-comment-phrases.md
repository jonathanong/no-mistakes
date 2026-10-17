# `no-mistakes/no-comment-phrases`

## Why

A comment that records a path as kept, retained, or preserved for compatibility is
often a tombstone: the old path should have been deleted and its callers
updated. Phrase lists are project policy, so this rule matches only configured
regular expressions. It has no built-in phrases.

## Disallowed

With the compatibility pattern from Options, line and block comments that match
are findings. The same words inside a string or template are not:

```ts
// kept for compatibility
/* preserved for backward compatibility */
// compat helpers we left in place
```

## Allowed

Descriptions that do not match the configured expression, and text that is not
a comment, are allowed. A genuine external protocol constraint stays only with
a suppression whose reason is nonempty when `requireSuppressionReason` is on:

```ts
// S3-compatible endpoint
const label = "kept for compatibility";
const query = `preserved for backward compatibility ${version}`;
// no-mistakes-disable-next-line no-comment-phrases -- external wire protocol still requires this field
// kept for compatibility with the published event schema
```

## Options

The rule is opt-in. Scope it with ordinary ESLint flat-config `files` or an
oxlint override. There is no default phrase list: omitted or empty `patterns`
report nothing.

`patterns` has at most 64 entries. Each entry requires `pattern` (a RegExp
source) and may set `flags` and a scoped `message`. The top-level `message` is
the fallback when an entry has no `message`. Patterns are compiled once per
file. `flags` is optional. When set, it must be a string of JS RegExp flags
(`d`, `g`, `i`, `m`, `s`, `u`, `v`, `y`), each at most once, in any order; an
empty string is allowed. A repeated flag or a character outside that set, such
as `gg` or `z`, is rejected when the config loads, in both ESLint and Oxlint,
instead of once per linted file. An invalid pattern source still produces one
`invalidPattern` diagnostic and that entry is skipped; valid entries still run.

`g` and `y` do not change how many times a comment is reported. Each comment is
tested once, with the expression's match index reset to 0 before and after the
test, so a global expression cannot skip a later comment and a zero-width match
cannot loop. A successful match, including a zero-width match such as `a*` or
`(?=alpha)`, reports that comment once per matching pattern. Sticky `y` matches
only at the start of the comment value (the text inside the delimiters, not
`//` or `/*`). `g` does not emit one report per match inside a comment.

Only line and block comment tokens are inspected. Shebang comments, string
literals, template literals, and regex literals are ignored. The matched text
is the comment value, so a leading space after `//` is part of the text.
Without the `s` flag, `.` does not cross a newline inside a block comment.

`requireSuppressionReason` defaults to off. Omitted or `false`, file, line, and
next-line directives suppress this rule exactly as they do for every other
rule, with or without a reason. `true` applies only to this rule: a directive
suppresses only when the reason after `--` on that same directive line is
nonempty after trimming. Other rules do not read this option.

```js
{
  files: ['src/legacy/**/*.js'],
  rules: {
    'no-mistakes/no-comment-phrases': ['error', {
      patterns: [
        {
          pattern: '(?:retain|kept|keep|preserve|left).{0,60}compat|compat.{0,60}(?:retain|kept|keep|preserve|left)|backwards?[- ]?compat',
          message: 'Delete the compatibility tombstone and update its callers.'
        },
        { pattern: 'configured-fallback-phrase' }
      ],
      message: 'Fallback configured phrase message.',
      requireSuppressionReason: true
    }]
  }
}
```

A pattern `message` scopes the diagnostic to that phrase. Findings use the
comment token's starting line, which is also the line suppression checks.

## Fix

Delete the replaced path and update its callers. Keep a comment only when it
records a real external constraint, and suppress that comment with a reason.
The rule does not rewrite comments or code.

## Suppression

```ts
// no-mistakes-disable-next-line no-comment-phrases -- external wire protocol still requires this field
// kept for compatibility with the published event schema
```

`no-mistakes-disable-line` and a top-of-file `no-mistakes-disable-file` (before
the first code token, including after a shebang) work the same way. The reason
is the text after the first `--` on the directive line. A missing `--`, a bare
`--`, or a whitespace-only reason does not count when `requireSuppressionReason`
is true, and the finding remains. When the option is off, those directives
still suppress. A directive for another rule does not suppress this one.
Directive-looking string literals never suppress. Native ESLint suppression
remains available when running ESLint.

The suppression comment is itself a comment. If its text matches a pattern, it
is reported unless a directive covers that comment's starting line. Put the
protocol rationale in the reason without repeating the banned phrase, or use
`no-mistakes-disable-line` on that comment.

## Related rules

- [`no-argv-main-guard`](no-argv-main-guard.md) is another opt-in rule. It uses
  the same directives and does not require a suppression reason.
- [`gh-api-argument-safety`](gh-api-argument-safety.md) documents the same file,
  line, and next-line directives without a reason requirement.
