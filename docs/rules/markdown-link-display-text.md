# `markdown-link-display-text`

Requires local Markdown link text to match the linked file basename when the
visible label looks like a Markdown filename.

## Why and when

Use this rule when Markdown link labels should remain searchable, accurate file
names instead of stale filenames.

## What it catches

It reports local Markdown links whose display text looks like a filename (it
ends with a configured extension and has no whitespace or slashes) but does not
match the target basename.

## Options

`extensions` is the only rule-local option. It defaults to `.md`; set it to a
complete extension list such as `[.md, .mdx]` when MDX should also be checked.
Generic rule `include` and `exclude` filters select files separately; there is
no rule-local `allow` option.

```yaml
rules:
  - rule: markdown-link-display-text
    scope: repository
    options:
      extensions: [.md, .mdx]
```

## Valid example

`[feature-parity.md](feature-parity.md)` passes because its visible label names
the destination. Descriptive prose such as `[Why](why.md)` is ignored.

```md
[feature-parity.md](feature-parity.md)
```

## Counterexample

`[SOURCE-STORIES.md](news-story-clusters.md)` points to a different basename
than the visible filename-style link text.

```md
[SOURCE-STORIES.md](news-story-clusters.md)
```

## Fix

Rename the link text to `news-story-clusters.md`, rename the target to match
the existing text, or use descriptive link text when the destination is not a
local Markdown file.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](README.md#how-analysis-works)); this rule has no unanalyzable inputs because it checks parsed Markdown link text, so `options.unanalyzable` has no effect on it.

## Suppression

Suppress only when the filename-style text is intentionally different from the
destination basename. The rule ignores images, fenced code, non-local link
destinations, and links that already use descriptive prose instead of a
filename-like label.

## Related rules

[`markdown-child-links`](markdown-child-links.md) requires missing links;
[`markdown-reachability`](markdown-reachability.md) validates their graph.
