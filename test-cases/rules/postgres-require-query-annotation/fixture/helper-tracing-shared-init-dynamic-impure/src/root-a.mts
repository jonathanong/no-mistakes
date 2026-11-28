import { query } from "@bench/db";
import { statement } from "./dynamic-impure.mjs";

export function runA() {
  return query(statement());
}
