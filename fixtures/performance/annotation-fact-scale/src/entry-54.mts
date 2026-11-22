import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read54() {
  invoke03(() => query("/* entry_54 */ SELECT 54"), payload);
  return query("SELECT id FROM items WHERE id = 54"); // expected-finding
}
