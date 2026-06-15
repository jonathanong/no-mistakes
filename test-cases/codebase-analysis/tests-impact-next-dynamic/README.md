# tests-impact-next-dynamic

Regression fixture proving `tests impact` traverses `next/dynamic()` import
boundaries: the inner `import('./foo.mts')` is captured as a `DynamicImport`
edge, so changing `foo.mts` surfaces `caller.test.mts` at `medium` confidence
with a `dynamic-import` warning.

- `foo.mts` — dynamically imported target.
- `caller.mts` — `dynamic(() => import('./foo.mts'))` boundary.
- `caller.test.mts` — test importing the caller.
