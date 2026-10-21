import { query } from "@example/db";
import { config } from "./config";

export function list() {
  return query(`SELECT id FROM ${config.schema}.users`);
}
