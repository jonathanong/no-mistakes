# `assert-no-fetch`

Reports a React component whose render path fetches, including a fetch inherited from a rendered child.

## Why and when

Use this check when a frontend tree must keep `fetch` out of components so data loading stays in a route or loader. The check stays off until `reactTraits.assertNoFetch` is true or `react check` is passed `--assert-no-fetch`.

## What it catches

A component is a violation when its own fetch facts are non-empty or a child aggregate has a fetch. The finding names the component and, when the extractor recorded one, the first fetch shape:

```text
app/components/Fetcher.tsx#Fetcher: assert-no-fetch GET /api/users
```

A `rules` entry for this id does not turn the check on. `react check` and aggregate `check` read `reactTraits.assertNoFetch`, or the CLI and Node `assertNoFetch` flag. The default is off.

## Options

There are no rule-local options. An unknown key under `options` fails configuration validation. Shared `effort` and `unanalyzable` keys are accepted and do not enable the check.

```yaml
rules:
  - rule: assert-no-fetch
    scope: repository
reactTraits:
  frontendRoot: app
  assertNoFetch: true
```

`assertNoFetch` defaults to false. `frontendRoot` selects the tree the standalone React command analyzes. The `rules` entry above only reserves the id.

## Valid example

A component that renders data it was given and does not call `fetch`.

```tsx
export default function Users({ users }) {
  return (
    <ul>
      {users.map((user) => (
        <li key={user.id}>{user.name}</li>
      ))}
    </ul>
  );
}
```

## Counterexample

```tsx
export default async function Fetcher() {
  const response = await fetch("/api/users");
  return <div>{response.status}</div>;
}
```

A parent that renders this component is also a violation, because the child aggregate still has a fetch.

## Fix

Move the fetch to the route or loader that owns the request, and pass the loaded data in as props. Removing the call from the parent is not enough while a rendered child still fetches.

## Suppression

Use `no-mistakes-disable-next-line assert-no-fetch` on the fetch, or `no-mistakes-disable-file` at the top of a generated component. Prefer moving the fetch.

## Related rules

[`server-route-client-boundary`](server-route-client-boundary.md) keeps client helpers out of configured server route folders. This check is the React-tree fetch ban. The command is [`react check`](../cli/react-check.md).
