import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read40() {
  invoke03(() => query("/* entry_40 */ SELECT 40"), payload);
  return query("SELECT id FROM items WHERE id = 40"); // expected-finding
}
