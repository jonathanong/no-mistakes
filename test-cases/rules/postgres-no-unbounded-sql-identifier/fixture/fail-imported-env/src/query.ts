import { query } from "@example/db";
import { env } from "node:process";

export function list() {
  return query(`SELECT id FROM ${env.SCHEMA}.users`);
}
