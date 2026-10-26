import sql from "sql-template-strings";
import { query, sql as splice } from "@example/db";
const fragment = sql`FROM hidden`;
const known = "WHERE secret";
const q = sql`SELECT ${known}`;
q.append(sql` WHERE id = ${fragment}`).append(sql` AND n = ${sql`SELECT 9`}`);
query(q);
const conditional = sql`SELECT ${known}`;
if (flag) conditional.append(sql` WHERE id = ${fragment}`);
query(conditional);
// Configured tags retain their established fragment-splicing contract.
const configured = splice`SELECT ${splice`1`}`;
if (flag) configured.append(" /* suffix */");
query(configured);
query(sql`SELECT 7`.append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */").append(" /* link */"));
// Evaluating an interpolated mutation still changes the separately consumed builder.
const mutated = sql`SELECT 8`;
query(sql`SELECT ${mutated.append(" WHERE active")}`);
query(mutated);
function valuesOnly() {
  const fragment = sql`FROM hidden`;
  return sql`SELECT ${fragment}`;
}
query(valuesOnly());
