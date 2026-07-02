# `markdown-link-display-text`

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
