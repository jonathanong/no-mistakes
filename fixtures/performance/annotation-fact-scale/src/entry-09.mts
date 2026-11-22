import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read09() {
  invoke03(() => query("/* entry_09 */ SELECT 9"), payload);
  return query("SELECT id FROM items WHERE id = 9"); // expected-finding
}
