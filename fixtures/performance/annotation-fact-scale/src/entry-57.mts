import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read57() {
  invoke03(() => query("/* entry_57 */ SELECT 57"), payload);
  return query("SELECT id FROM items WHERE id = 57"); // expected-finding
}
