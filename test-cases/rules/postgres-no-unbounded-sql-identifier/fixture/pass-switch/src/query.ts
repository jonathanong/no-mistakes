import { query } from "@example/db";
import { schemaOf } from "./schema";

export function list() {
  return query(`SELECT id FROM ${schemaOf(process.env.SCHEMA ?? "")}.users`);
}
