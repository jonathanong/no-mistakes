import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read02() {
  invoke03(() => query("/* entry_02 */ SELECT 2"), payload);
  return query("SELECT id FROM items WHERE id = 2"); // expected-finding
}
