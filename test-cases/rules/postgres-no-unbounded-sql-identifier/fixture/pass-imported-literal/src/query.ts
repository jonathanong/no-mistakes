import { query } from "@example/db";
import { schema } from "./schema";

export function list() {
  return query(`SELECT id FROM ${schema.name}.users`);
}
