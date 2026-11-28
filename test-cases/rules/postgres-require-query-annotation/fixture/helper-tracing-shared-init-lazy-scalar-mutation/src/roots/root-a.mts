import { query } from "@bench/db";
import { prefix } from "../shared/state.mjs";
import { makeSql } from "../shared/middle.mjs";

export const seedPrefix = prefix;

export function handleA() {
  return query(makeSql("SELECT 1"));
}
