import { it } from "vitest";
import { query } from "./query-files.mjs";
it.concurrent("query after cleanup", async () => { await Promise.resolve(); await query("table"); });
it.concurrent("finishes first", () => {});
