import { query } from "@example/db";
import { firstHelperSql, secondHelperSql } from "./lock-clause.mts";

export function chooseHelper(flag: boolean) {
  return query(flag ? `${firstHelperSql}` : `${secondHelperSql}`);
}
