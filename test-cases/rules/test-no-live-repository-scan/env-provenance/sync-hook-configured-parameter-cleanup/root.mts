import { it } from "vitest";
import { query } from "./query-files.mjs";
it.concurrent("query after cleanup", async () => { await Promise.resolve(); await query(process.env.X); });
it.concurrent("finishes first", () => {});
