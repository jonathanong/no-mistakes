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

`excludes` is an optional list of route folders or files to leave outside the
boundary. Entries may be globs, root-relative paths, or literal path
components. It defaults to an empty list. Configure the target server project
and its route globs separately; the rule does not infer a client directory
convention.

## Valid example

```yaml
# .no-mistakes.yml
projects:
  api:
    type: server
    root: backend
    routes: ['api/**']
rules:
  - rule: server-route-client-boundary
    projects: [api]
```

```ts
// backend/api/users/route.ts
import express from 'express';
const app = express();
app.get('/api/users', (_request, response) => response.json([]));
```

```ts
// backend/services/users-client.ts
import axios from 'axios';
export async function getUsers() {
  return axios.get('/api/users');
}
```

## Counterexample

```yaml
# .no-mistakes.yml
projects:
  api:
    type: server
    root: backend
    routes: ['api/**']
rules:
  - rule: server-route-client-boundary
    projects: [api]
```

```ts
// backend/api/users/route.ts
import express from 'express';
const app = express();
app.get('/api/users', (_request, response) => response.json([]));
```

```ts
// backend/api/users/client.ts
import axios from 'axios';
export async function getUsers() {
  return axios.get('/api/users');
}
```

The resolved Express route establishes the folder boundary; the Axios call
in its sibling file violates it.

## Fix

Move the client/helper into the configured service/client area, or narrow the
route project if this directory is not actually a server route boundary.

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Selected files that cannot be read as UTF-8 produce an `unanalyzable`
diagnostic instead of silently passing. Restore read access or repair the
encoding so the rule can verify the file. The shared `options.unanalyzable`
setting inherits the repository policy: `report` (the default) fails,
`advise` emits non-failing advice, and `ignore` omits uncertainty. Definite
violations remain failures under every policy, including when `effort: low`
falls back to `max`.

Configured scope, includes, excludes, and rule-specific exemptions still
select the inputs. Suppressions apply to both channels, including directives
retained in an invalid UTF-8 read error; source bytes are never reinterpreted
as analyzable code. Rule-level custom messages apply to these diagnostics.

TS/JS parse failures use the same policy for configured route candidates or
files under a verified route folder. The prepared runner consumes its existing
read/parse errors and boundary facts; it does not read or parse files again to
recover missing facts. The standalone rule reports the same failures.

## Suppression

Use a narrower project or route glob for intentional mixed folders. A file
directive is appropriate only for a legacy route-local helper with a documented
bundle boundary.

## Limitations

Discovery skips any directory segment named `node_modules`, `dist`, `.git`,
`.next`, `coverage`, `fixtures`, `target`, or `build`. A client call under
one of those names is not seen.

```ts
// backend/api/users/build/client.ts
fetch("/users");
```

## Related rules

[`required-entrypoint-reachability`](required-entrypoint-reachability.md)
checks runtime registration; [`unique-exports`](unique-exports.md) checks
cross-file API-name collisions.
