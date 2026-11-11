import { it } from "vitest";
import { query } from "./query-files.mjs";
import { afterReset } from "./setup.mjs";
async function queryAfterReset() { await afterReset(); await query("table"); }
it.concurrent("query after reset", queryAfterReset);
it.concurrent("finishes first", () => {});
