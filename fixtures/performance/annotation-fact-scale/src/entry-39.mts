import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read39() {
  invoke03(() => query("/* entry_39 */ SELECT 39"), payload);
  return query("SELECT id FROM items WHERE id = 39"); // expected-finding
}
