import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read05() {
  invoke03(() => query("/* entry_05 */ SELECT 5"), payload);
  return query("SELECT id FROM items WHERE id = 5"); // expected-finding
}
