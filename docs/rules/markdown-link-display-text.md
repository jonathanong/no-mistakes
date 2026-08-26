# `markdown-link-display-text`

## Why and when

Use this rule when Markdown link labels should remain searchable, accurate file
names instead of stale descriptions.

## What it catches

It reports local Markdown links whose display text does not match the target
basename under the configured normalization rules.

## Options

`extensions` is the only rule-local option. It defaults to `.md`; set it to a
complete extension list such as `[.md, .mdx]` when MDX should also be checked.
Generic rule `include` and `exclude` filters select files separately; there is
no rule-local `allow` option.

## Valid example

`[feature-parity.md](feature-parity.md)` passes because its visible label names
the destination.

## Related rules

[`markdown-child-links`](markdown-child-links.md) requires missing links;
[`markdown-reachability`](markdown-reachability.md) validates their graph.

Requires local Markdown link text to match the linked file basename.

```yaml
rules:
  - rule: markdown-link-display-text
    projects: [web]
    options:
      extensions: [.md, .mdx]
```

Counterexample: `[SOURCE-STORIES.md](docs/news-story-clusters.md)` points to a
different basename than the visible link text.

Fix: rename the link text to `news-story-clusters.md`, rename the target to
match the existing text, or use descriptive link text when the destination is
not a local Markdown file.

Suppression caveat: suppress only when the filename-style text is intentionally
different from the destination basename. The rule ignores images, fenced code,
non-local link destinations, and links that already use descriptive prose
instead of a filename-like label.
