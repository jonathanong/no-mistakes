import { query, sql } from "@example/db";
const tail = flag
  ? sql`WHERE id = ${first} \
 OFFSET 1`
  : sql`WHERE id = ${second}
 LIMIT 1`;
query(sql`SELECT ${a}, ${b}, ${c}, ${d}, ${e}, ${f}, ${g}, ${h}, ${i} FROM users ${tail} AND owner = ${owner} AND sql_placeholder_10 = 0`);
