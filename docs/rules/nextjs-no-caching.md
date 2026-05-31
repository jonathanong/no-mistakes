# `nextjs-no-caching`

Bans Next.js caching features such as cache directives, cache wrappers, and
cache-related fetch/config settings.

```yaml
rules:
  - rule: nextjs-no-caching
    projects: [web]
```

Counterexample: `fetch(url, { cache: "force-cache" })`.

Fix: remove caching or isolate it behind an explicitly allowed architecture.
