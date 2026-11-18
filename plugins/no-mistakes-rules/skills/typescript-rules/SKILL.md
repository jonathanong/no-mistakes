---
name: typescript-rules
description: "Load before writing or reviewing TypeScript, JavaScript, or Node.js code: async functions and promises, error handling, HTTP clients (fetch, undici, axios), Redis or Valkey caches, process lifecycle, package boundaries. Lists the defects to avoid and the patterns that need a stated reason."
---

# TypeScript and Node rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

## Async

- MUST NOT leave a floating promise from an async boundary; `await` it, or `void` it with a rejection handler.
- MUST NOT swallow rejections with `.catch(() => {})`; log, report, transform, or rethrow.
- MUST NOT `return promise` inside `try` when the `catch` should handle it; without `await` the rejection escapes the `catch`. Use `return await`.
- MUST NOT `await` a synchronous array method.
- MUST NOT fan out with unbounded `Promise.all` over input-sized lists; bound concurrency or batch.
- MUST NOT call network, DNS, or model clients without an explicit timeout or abort signal.
- MUST NOT display stale async results; abort superseded requests or compare tokens.
- SHOULD NOT issue consecutive independent awaited reads; batch the key set into one call.

## Values and errors

- MUST NOT collapse an explicit `null` into a default with `??` or `||` when null has meaning; test `=== undefined`.
- MUST NOT mutate imports, module bindings, or parameters; build and return a new value.
- MUST NOT drop `cause` when wrapping an error.
- MUST NOT catch every error and return `null`; return null only for expected statuses.
- MUST NOT compare, hash, lock, or cache identities in non-normalized form.
- MUST NOT reuse one variable, key, or flag across concurrent calls so a later call overwrites an earlier one.
- MUST NOT add an option without forwarding it through every caller and helper.

## Static analysis and dead code

- MUST NOT hide calls behind computed keys, runtime registries, or forwarding lambdas; static analysis cannot trace them. Call the target by name.

## Process and IO

- MUST NOT call `process.exit()` or `process.kill()` in runtime code; register graceful shutdown.
- MUST NOT register lifecycle callbacks conditionally or lazily in owner modules; register at top level.
- MUST NOT detect the entry module by comparing `process.argv[1]`; use the runtime's main-module check.
- MUST NOT buffer large bodies or files in memory; stream them.
- MUST NOT do blocking or CPU-heavy work on the event loop; use worker threads or async native calls.
- MUST NOT read disk, spawn processes, or reparse inside a hot loop; hoist and reuse.
- SHOULD NOT walk a repository without `.gitignore` awareness.
- MUST NOT add a second logger or error reporter; use the existing ones, and do not report expected 4xx as errors.

## Redis and Valkey

- MUST NOT send sequential commands for a known key set; use `MGET`, a pipeline, or a script.
- MUST NOT read replicas for auth or rate limits.
- MUST NOT leave pub/sub subscriptions open after the client disconnects.
- SHOULD NOT `DEL` large keys; use `UNLINK`.

## Packages and boundaries

- MUST NOT import a package the owning `package.json` does not declare.
- MUST NOT use relative imports that escape a package root.
- MUST NOT use Node-only APIs in code that other runtimes (Workers, browsers) load.
- MUST NOT leave tracked TS files outside every `tsconfig`, or create workspace dependency cycles.
