import { read } from "@example/db";
import sql from "sql-template-strings";

export async function helper(include: boolean) {
  const query = buildQuery();
  if (include) query.append(sql` WHERE execution.id IS NOT NULL`);
  query.append(sql` ORDER BY execution.id LIMIT 10`);
  await read(query);
}

function buildQuery() {
  return sql`SELECT execution.id FROM accounts a
    JOIN LATERAL (
      SELECT * FROM accounts attempt
      WHERE attempt.id = a.id
      ORDER BY attempt.id DESC LIMIT 1
    ) execution ON true
    UNION ALL
    SELECT execution.id FROM accounts a
    JOIN LATERAL (
      SELECT * FROM accounts attempt
      WHERE attempt.id = a.id
      ORDER BY attempt.id DESC LIMIT 1
    ) execution ON true`;
}
