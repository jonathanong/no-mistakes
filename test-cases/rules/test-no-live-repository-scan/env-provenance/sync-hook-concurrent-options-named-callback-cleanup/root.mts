import { it } from "vitest";
import { query } from "./query-files.mjs";
import { afterReset } from "./setup.mjs";
const queryAfterReset = async () => { await afterReset(); await query("table"); };
it("query after reset", { concurrent: true }, queryAfterReset);
it.concurrent("finishes first", () => {});
