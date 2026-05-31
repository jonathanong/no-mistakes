# `server-route-client-boundary`

Keeps client/helper code out of server route folders.

```yaml
rules:
  - rule: server-route-client-boundary
    projects: [api]
```

Counterexample: `backend/api/users/client.ts` beside route definitions.

Fix: move clients to an allowed client/service directory and keep route folders
focused on route definitions.
