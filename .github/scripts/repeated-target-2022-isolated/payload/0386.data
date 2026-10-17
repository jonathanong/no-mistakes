import { read } from "@example/db";
import sql from "sql-template-strings";

const values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

export async function helper(include: boolean) {
  const query = buildQuery();
  if (include) query.append(sql` AND execution.id = ${values[9]}`);
  query.append(sql` UNION ALL SELECT execution.id FROM accounts a
    JOIN LATERAL (
      SELECT * FROM accounts second_attempt
      WHERE second_attempt.id = a.id
    ) execution ON true`);
  await read(query);
}

function buildQuery() {
  return sql`SELECT execution.id FROM accounts a
    JOIN accounts filtered ON filtered.id IN (${values[0]}, ${values[1]}, ${values[2]}, ${values[3]}, ${values[4]}, ${values[5]}, ${values[6]}, ${values[7]}, ${values[8]})
    JOIN LATERAL (
      SELECT * FROM accounts first_attempt
      WHERE first_attempt.id = a.id
    ) execution ON true`;
}
