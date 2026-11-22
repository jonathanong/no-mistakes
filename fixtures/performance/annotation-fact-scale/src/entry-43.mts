import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read43() {
  invoke03(() => query("/* entry_43 */ SELECT 43"), payload);
  return query("SELECT id FROM items WHERE id = 43"); // expected-finding
}
