import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read41() {
  invoke03(() => query("/* entry_41 */ SELECT 41"), payload);
  return query("SELECT id FROM items WHERE id = 41"); // expected-finding
}
