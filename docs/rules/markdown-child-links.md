# `markdown-child-links`

Requires every Markdown file matching a child glob to be linked from a
configured parent Markdown file. Use this when a folder README (or similar
index) must list its sibling documents.

```yaml
rules:
  - rule: markdown-child-links
    scope: repository
    options:
      groups:
        - parents: ["docs/**/README.md"]
          children: ["docs/**/*.md", "docs/*.md"]
          requireWholeFile: true
```

Links are resolved from shared Markdown facts (`link_destinations`). External
URLs are ignored. The parent file itself is not required to link to itself.
`requireWholeFile: true` ignores destinations that include a `#` fragment so a
section link does not count as covering the child file.

Counterexample: `docs/README.md` exists and `docs/guide.md` matches the child
glob, but the README has no local link to `guide.md`.

```md
# Docs
```

Fix: add a whole-file relative link from a parent:

```md
# Docs

- [Guide](guide.md)
```

Use `no-mistakes-disable-next-line markdown-child-links` or
`no-mistakes-disable-file` when a child is intentionally unlisted.
