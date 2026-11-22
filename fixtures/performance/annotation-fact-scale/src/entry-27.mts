import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read27() {
  invoke03(() => query("/* entry_27 */ SELECT 27"), payload);
  return query("SELECT id FROM items WHERE id = 27"); // expected-finding
}
