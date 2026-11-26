import { runQuery } from "@acme/psql";
import { runMiddleSql } from "../shared/top.mjs";

export const boundC = runMiddleSql;

export function handleC() {
  return runQuery(boundC("receipts"));
}
