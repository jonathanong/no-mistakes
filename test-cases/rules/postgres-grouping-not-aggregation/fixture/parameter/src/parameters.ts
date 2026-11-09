import { query, sql } from "db";

export async function parameterCollision(value: string) {
  await query(sql`
    SELECT COUNT(DISTINCT sql_placeholder_1) /* CASE: embedded-real-nullable-unique */
    FROM marker_inputs GROUP BY sql_placeholder_1
  `);
  await query(sql`
    SELECT COUNT(DISTINCT ${value}) /* CASE: embedded-bind-collision-negative */
    FROM marker_inputs GROUP BY sql_placeholder_1
  `);
}

export async function parameterCollisionAppend(value: string) {
  const positive = sql`SELECT COUNT(DISTINCT sql_placeholder_1) /* CASE: append-real-nullable-unique */ `;
  positive.append(sql`FROM marker_inputs GROUP BY sql_placeholder_1`);
  const negative = sql`SELECT COUNT(DISTINCT ${value}) /* CASE: append-bind-collision-negative */ `;
  negative.append(sql`FROM marker_inputs GROUP BY sql_placeholder_1`);
  await query(positive);
  await query(negative);
}
