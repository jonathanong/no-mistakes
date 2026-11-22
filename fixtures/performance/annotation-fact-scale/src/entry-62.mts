import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read62() {
  invoke03(() => query("/* entry_62 */ SELECT 62"), payload);
  return query("SELECT id FROM items WHERE id = 62"); // expected-finding
}
