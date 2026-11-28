# `no-mistakes/server-require-nullable-fetch-wrapper`

## Why

Nullable entity getters need one configured wrapper so a missing entity maps to
the same `null` or response behavior in every server handler.

## Disallowed

```ts
// backend/users.ts
type MaybeUser = { id: string } | null;

export async function getUserRoute(id: string): Promise<MaybeUser> {
  return getUserById(id);
}
```

## Allowed

```ts
// backend/users.ts
type MaybeUser = { id: string } | null;

export async function getUserRoute(id: string): Promise<MaybeUser> {
  return nullableEntity(getUserById(id));
}
```

## Options

The schema accepts an options object.

```js
{
  rules: {
    "no-mistakes/server-require-nullable-fetch-wrapper": ["error", {
      getterCalleePatterns: ["^getUserById$"],
      requiredWrapperCallee: "nullableEntity",
      nullableReturnTypeNames: ["MaybeUser"],
      includePathPatterns: ["backend/**"]
    }]
  }
}
```

- `getterCalleePatterns` and `requiredWrapperCallee` are required.
- `includePathPatterns` and `excludePathPatterns` scope matching files.
- `nullableReturnTypeNames` adds nullable return-type hints.
- `inferNullableFromTopLevelEntityPath` and `topLevelEntityPathPatterns`
  enable path-based nullable inference.

Optional lists default to empty and inference defaults to `false`.

`unanalyzable` is `"finding"` or `"ignore"`. It defaults to `"finding"`. A
getter or wrapper whose import cannot be resolved reports `unanalyzable`
instead of a definite `wrapper` finding or a silent pass. `"ignore"` drops
only that uncertain report, including a known getter nested in an unresolved call. A known member that matches no getter pattern stays
quiet. An unresolved call between a getter and its wrapper does not hide that
wrapper.

An import rename is spelled back to the imported name before the getter pattern is tested. `require("./api")`, `import Api = require("./api")`, and `import * as api` spell the member the same way. `require("./api").getWidget("1")` is that getter, and so is `const read = require("./api").getWidget`. `import * as api from "./data"; const read = api.getWidget` is that getter too. `require("./data").api.getWidget("1")` matches `api.getWidget`, and so does `const getWidget = require("./data").api.getWidget` and `const group = require("./data").api; const read = group.getWidget`. `const svc = require("./data").services.api; svc.getWidget("1")` keeps `services.api.getWidget`. `require(moduleName).api.getWidget("1")` is unanalyzable. A getter pattern the rule cannot parse, such as `api.(getWidget|getGadget)`, stays unanalyzable instead of proving the member is not a getter. `var api = require("./data")` followed by `const { getWidget: read } = api` is that getter too, and a later write to `api` is unanalyzable. `const { api: { getWidget: read } } = require("./data")` is `api.getWidget`, and a nested pattern with no default stays that member. `const { api: { getWidget: read } = fallback } = require("./data")` reports `unanalyzable` when `fallback` is not that `api` member, including a default whose qualified path differs. A default on the bound name, such as `getWidget: read = missing`, reports `unanalyzable` when that default is not the same import. A type alias or type parameter named `require` does not hide the loader. `let read; read = require("./data").getWidget` is that getter. `let { read }` or `let [read]` later assigned that getter is the same getter, and a destructure that is never assigned an import stays quiet. A destructure of a local value does not make a later write of that value unanalyzable. `let read = target` before `target` is assigned an import is not that import, and a later `read = target` reports `unanalyzable`. `({ getWidget: read } = require("./data"))` is that getter, and a destructuring assignment whose member cannot be resolved reports `unanalyzable`. An assignment that is not an import stays quiet, and a second write is unanalyzable. `require(moduleName)()` and `require?.(moduleName)("1")` are unanalyzable. A dynamic module or member stays unanalyzable, including `const { getWidget } = require(moduleName)` and `api.getWidget.call` when `api` itself cannot be resolved. `const { api: client } = require(moduleName); client.getWidget("1")` is unanalyzable when a getter pattern ends in `getWidget`, and `client.other("1")` stays quiet. `api[method]("1")` after `import * as api` reports unanalyzable because the member has no static name. `import type * as api` does not. `const api = data.default` followed by `api.getWidget("1")` spells `default.getWidget`, and so does `data.default.getWidget("1")`. `const { getWidget: read } = api` spells `default.getWidget`, and the same destructure of the namespace itself stays `getWidget`. An ordinary default import keeps the member name without that prefix. `api["getWidget"]("1")` stays that getter. `load().getWidget("1")` and `items[method]("1")` stay quiet when `load` and `items` are not imports. `wrapNullable(getWidget())` is not inside the wrapper when `wrapNullable` is `require(moduleName)` or `const { wrapNullable } = require(moduleName)`, so that getter reports unanalyzable. `require("./wrap").wrapNullable(getWidget())` is still inside. `"ignore"` drops that report instead of falling through to a definite wrapper finding. A local function named `require` is not the loader. An optional call, a comma callee, `satisfies` or `as` around the exported function or the receiver, and `.call` or a literal-array `.apply` inside the wrapper are the same call. `import * as data from "./data"; data.api.getWidget("1")` spells `api.getWidget`. `import { api } from "./data"; const { getWidget: read } = api` is that getter too.

## Fix

Wrap the configured getter within the same function boundary using the required
wrapper, or narrow the configured getter/path policy.

## Limits

A callee whose own name is `call` or `apply`, such as `rpc.call("/users/1")` or `rpc.call(...args)`, is matched as that callee before forwarding is considered. `requiredWrapperCallee` matches the local binding, so `import { nullableEntity as wrap }` with `requiredWrapperCallee: "wrap"` still accepts `wrap(getUser())`. The imported name `nullableEntity` matches as well. A getter pattern does the same: `import { lookup as getUser }` matches both `^getUser$` and `^lookup$`.

Another file's re-export is not visible here. `import { get } from "./barrel.mjs"` stays unreported when `barrel.mjs` is `export { serverApi as get } from "./server-api"` or `export * from "./server-api"`. A parameter default such as `function run({ get } = serverApi)` is not judged, because the caller can pass a different object. `.apply` with a non-literal argument list is not rewritten. `nullableEntity.call(serverApi.get("/users/1"))` and `nullableEntity.apply(serverApi.get("/users/1"), [])` use the getter only as `this`, so the fetch is still unwrapped, including when that wrapper binding is unresolved. An unresolved receiver that is not the wrapper, such as `rpc.call(getWidget())`, stays unanalyzable. A forwarded argument of an unresolved wrapper stays `unanalyzable`. `nullableEntity.call(null, serverApi.get("/users/1"))` passes the getter through.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/server-require-nullable-fetch-wrapper -- handler deliberately maps absence to 404 itself
return getUserById(id);
```

`no-mistakes-disable-line`, `no-mistakes-disable-next-line`, and
`no-mistakes-disable-file` also suppress this rule when the comment names it.
Directive-looking string literals do not.

## Related rules

- [`no-global-fetch-outside-helper`](no-global-fetch-outside-helper.md)
  centralizes another server I/O boundary.
