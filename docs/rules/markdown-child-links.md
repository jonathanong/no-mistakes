# `markdown-child-links`

## Why and when

Use this rule when a README or index must be a complete navigational inventory
for a directory of Markdown children.

## What it requires

It requires configured parent documents to link each matching child, including
the supported canonical HTML-list form.

## Options

`groups` is the only rule-local option. Each group contains `parents`,
`children`, and optional `requireWholeFile` and
`countCanonicalHtmlListItems` booleans. Both booleans default to `false`, and
all lists default to empty; an empty group list performs no checks.
Shared rule `include`/`exclude` filters further scope Markdown files.

## Valid example

A parent list linking every matching child exactly once passes.

## Related rules

[`markdown-reachability`](markdown-reachability.md) verifies a document can be
found from instruction roots; [`markdown-link-display-text`](markdown-link-display-text.md)
checks the quality of those links.

## Suppression

Use a next-line directive on the intentionally unlisted child or a file
directive only for an index outside the policy. Prefer narrowing child globs or
adding the missing link.

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
          countCanonicalHtmlListItems: true
```

Links are resolved from shared Markdown facts (`link_destinations`). External
URLs are ignored. The parent file itself is not required to link to itself.
`requireWholeFile: true` ignores destinations that include a `#` fragment so a
section link does not count as covering the child file. pulldown-cmark does not
see HTML list items such as `- <a id="guide"></a>[Guide](guide.md)`; set
`countCanonicalHtmlListItems: true` to count those as parent links.

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
