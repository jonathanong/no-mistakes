import { createRequire } from "node:module";
import sql from "@example/sql";
import type { Missing } from "./type-only-broken";

const load = createRequire(import.meta.url);
load("./created-broken");
load.resolve("./path-only-broken");
require.resolve("./path-only-broken");

export function read() {
  return sql`select 1`;
}
