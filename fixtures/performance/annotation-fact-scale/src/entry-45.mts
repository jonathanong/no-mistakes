import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read45() {
  invoke03(() => query("/* entry_45 */ SELECT 45"), payload);
  return query("SELECT id FROM items WHERE id = 45"); // expected-finding
}
