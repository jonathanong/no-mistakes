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

## Why and when

Use this rule for server applications where route directories are deployment
boundaries and client code would blur ownership or accidentally enter a server
bundle.

## What it catches/requires

Files selected as client/helper code must not live beneath configured server
route directories. The project configuration supplies the route scope.

## Options and defaults

There are no rule-local options. Configure the target server project and its
route globs; the rule does not infer a client directory convention.

## Valid example

```text
backend/api/users/route.ts
backend/services/users-client.ts
```

## Counterexample

```text
backend/api/users/client.ts
```

## Fix

Move the client/helper into the configured service/client area, or narrow the
route project if this directory is not actually a server route boundary.

## Suppression

Use a narrower project or route glob for intentional mixed folders. A file
directive is appropriate only for a legacy route-local helper with a documented
bundle boundary.

## Related rules

[`required-entrypoint-reachability`](required-entrypoint-reachability.md)
checks runtime registration; [`unique-exports`](unique-exports.md) checks
cross-file API-name collisions.
