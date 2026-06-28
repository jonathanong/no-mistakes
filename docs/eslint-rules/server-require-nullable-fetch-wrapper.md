# `no-mistakes/server-require-nullable-fetch-wrapper`

Requires configured nullable entity fetch helpers to wrap getter calls in a
project wrapper.

Why: entity getters often throw, return undefined, or use transport-specific
missing-value behavior. A single wrapper keeps nullable helper contracts
consistent.

```ts
export function getUser(): User | null {
  return serverApi.get("/users/1");
}
```

Counterexample: an exported helper has a nullable return type and calls a
configured getter directly.

Fix: wrap the getter call in the configured nullable wrapper.

```ts
export function getUser(): User | null {
  return nullableEntity(serverApi.get("/users/1"));
}
```

Options: `includePathPatterns`, `excludePathPatterns`, `getterCalleePatterns`,
`requiredWrapperCallee`, `nullableReturnTypeNames`,
`inferNullableFromTopLevelEntityPath`, and `topLevelEntityPathPatterns`.
