import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read30() {
  invoke03(() => query("/* entry_30 */ SELECT 30"), payload);
  return query("SELECT id FROM items WHERE id = 30"); // expected-finding
}
