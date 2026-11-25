import { query } from "@bench/db";
import { statement } from "./dynamic-impure.mjs";

export function runB() {
  return query(statement());
}
