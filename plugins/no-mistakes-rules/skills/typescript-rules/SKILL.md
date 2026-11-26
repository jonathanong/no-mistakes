---
name: typescript-rules
description: "Load before writing or reviewing TypeScript, JavaScript, or Node.js code: async functions and promises, error handling, HTTP clients (fetch, undici, axios), Redis or Valkey caches, process lifecycle, package boundaries. Lists the defects to avoid and the patterns that need a stated reason."
---

# TypeScript and Node rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

## Async

- MUST NOT leave a floating promise from an async boundary; `await` it, or `void` it with a rejection handler.
- SHOULD NOT swallow rejections with `.catch(() => {})` unless the operation is explicitly best-effort and its failure needs no follow-up.
- MUST NOT `return promise` inside `try` when the `catch` should handle it; without `await` the rejection escapes the `catch`. Use `return await`.
- MUST NOT `await` a synchronous array method.
- SHOULD NOT use top-level `await` or top-level `await using` in library modules; module evaluation suspends and barrel re-exports can expose uninitialized bindings in some runtimes and bundlers.
- SHOULD NOT fan out with unbounded `Promise.all` over input-sized lists; bound concurrency or batch.
- MUST NOT call network, DNS, or model clients without an explicit timeout or abort signal.
- MUST NOT display stale async results; abort superseded requests or compare tokens.
- SHOULD NOT issue consecutive independent awaited reads; batch the key set into one call.

## Values and errors

- MUST NOT collapse an explicit `null` into a default with `??` or `||` when null has meaning; test `=== undefined`.
- MUST NOT reassign imported bindings.
- SHOULD NOT mutate shared module state or parameters unless the mutation is part of their contract.
- MUST NOT drop `cause` when wrapping an error.
- MUST NOT catch every error and return `null`; return null only for expected statuses.
- MUST NOT compare, hash, lock, or cache identities in non-normalized form.
- MUST NOT reuse one variable, key, or flag across concurrent calls so a later call overwrites an earlier one.
- SHOULD NOT expose an option that has no effect on a supported call path unless it is a documented compatibility option.

## Static analysis and dead code

- SHOULD NOT export an error class that nothing constructs or subclasses unless it is public library API or a runtime loader constructs it. Otherwise, specific error handlers can never match it.

## Process and IO

- SHOULD NOT call `process.exit()` or `process.kill()` in library or long-running service code when it interrupts pending work or bypasses cleanup.
- MUST NOT detect the entry module by comparing `process.argv[1]`; use the runtime's main-module check.
- MUST NOT buffer large bodies or files in memory; stream them.
- MUST NOT do blocking or CPU-heavy work on the event loop; use worker threads or async native calls.
- MUST NOT read disk, spawn processes, or reparse inside a hot loop; hoist and reuse.
- SHOULD NOT walk a repository without `.gitignore` awareness.
- SHOULD NOT add a second logger or error reporter unless it serves a separate purpose.
- SHOULD NOT report expected 4xx responses as application errors unless they indicate a failure the service must handle.

## Redis and Valkey

- MUST NOT send sequential commands for a known key set; use `MGET`, a pipeline, or a script.
- MUST NOT read replicas for auth or rate limits.
- MUST NOT leave pub/sub subscriptions open after the client disconnects.
- SHOULD NOT `DEL` large keys; use `UNLINK`.

## Packages and boundaries

- MUST NOT use relative imports that escape a package root.
- MUST NOT use Node-only APIs in code that other runtimes (Workers, browsers) load.
- SHOULD NOT leave tracked TypeScript files that need type checking outside every `tsconfig`.
- SHOULD NOT create workspace dependency cycles.
