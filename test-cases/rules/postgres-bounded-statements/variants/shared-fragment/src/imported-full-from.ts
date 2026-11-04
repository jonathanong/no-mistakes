import { query, sql } from "@example/db";
import { singleRootFragmentSql } from "./single-root-fragment";

export function firstImported() {
  return query(sql`SELECT id`.append(singleRootFragmentSql()));
}

export function secondImported() {
  return query(sql`SELECT id`.append(singleRootFragmentSql()));
}
