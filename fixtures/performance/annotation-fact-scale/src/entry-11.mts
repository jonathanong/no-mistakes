import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read11() {
  invoke03(() => query("/* entry_11 */ SELECT 11"), payload);
  return query("SELECT id FROM items WHERE id = 11"); // expected-finding
}
