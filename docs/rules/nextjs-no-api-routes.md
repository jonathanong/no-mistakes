# `nextjs-no-api-routes`

Bans Next.js API route files.

```yaml
rules:
  - rule: nextjs-no-api-routes
    projects: [web]
```

Counterexample: `pages/api/users.ts` or `app/api/users/route.ts`.

Fix: move API behavior to the configured backend/server project.
