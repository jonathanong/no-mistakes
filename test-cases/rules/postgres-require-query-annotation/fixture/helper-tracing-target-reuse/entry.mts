import { query } from "@app/db";
import { query as other } from "@app/other";
import { safe, unsafe, shared, ambiguous, missing, cycled, default as absentDefault } from "./barrel.mjs";
// Unused runtime and type imports must never cause resolver work.
import { unused } from "./unused.mjs";
import type { TypeOnly } from "./type-only.mjs";
query(safe()); // safe-query
query(unsafe()); // unsafe-query
query(shared()); // diamond-query
query(ambiguous()); // ambiguous-query
query(missing()); // missing-query
query(cycled()); // cycle-query
query(absentDefault()); // default-query
other(safe()); // other-safe-query
other(unsafe()); // other-unsafe-query
other(missing()); // other-missing-query
