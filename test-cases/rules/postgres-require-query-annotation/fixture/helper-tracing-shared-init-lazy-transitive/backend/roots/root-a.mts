import { runQuery } from "@acme/psql";
import { runMiddleSql } from "../shared/top.mjs";

export const boundA = runMiddleSql;

export function handleA() {
  return runQuery(boundA("orders"));
}
