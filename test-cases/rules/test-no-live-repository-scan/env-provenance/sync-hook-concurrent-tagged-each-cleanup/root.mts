import { it } from "vitest";
import { query } from "./query-files.mjs";
import { afterReset } from "./setup.mjs";
it.concurrent.each`name
${"query"}`("query after reset", async () => { await afterReset(); await query("table"); });
it.concurrent("finishes first", () => {});
