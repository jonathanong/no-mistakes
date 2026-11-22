import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read26() {
  invoke03(() => query("/* entry_26 */ SELECT 26"), payload);
  return query("SELECT id FROM items WHERE id = 26"); // expected-finding
}
