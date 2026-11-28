import { runQuery } from "@acme/psql";
import { runMiddleSql } from "../shared/top.mjs";

export const boundB = runMiddleSql;

export function handleB() {
  return runQuery(boundB("invoices"));
}
