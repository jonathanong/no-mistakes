import templateSql from "sql-template-strings";
import { query } from "@example/db";

const extra = ", other_col";
const blank = "";

export function list(col: string) {
  return query(templateSql`SELECT ${col}${extra} FROM widgets`);
}

export function bare(col: string) {
  return query(templateSql`SELECT ${col}${blank} FROM widgets`);
}
