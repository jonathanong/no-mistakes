import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read51() {
  invoke03(() => query("/* entry_51 */ SELECT 51"), payload);
  return query("SELECT id FROM items WHERE id = 51"); // expected-finding
}
