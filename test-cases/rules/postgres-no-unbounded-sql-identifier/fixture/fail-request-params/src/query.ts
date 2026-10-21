import { query } from "@example/db";

export function list(req: { params: { schema: string } }) {
  return query(`SELECT id FROM ${req.params.schema}.users`);
}
