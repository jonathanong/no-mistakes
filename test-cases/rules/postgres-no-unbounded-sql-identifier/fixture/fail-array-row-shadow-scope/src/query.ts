import { query } from "@example/db";
const result = await query(`SELECT schema FROM tenants`);
const [first] = result.rows;
{
  const first = { schema: "public" };
  first.schema = "public";
  query(`SELECT id FROM ${first.schema}.users`);
}
query(`SELECT id FROM ${first.schema}.users`);
// The finite inner binding must not prove anything about the outer row binding.
