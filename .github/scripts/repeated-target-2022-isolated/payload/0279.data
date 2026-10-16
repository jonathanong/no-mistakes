import { query, sql } from "@example/db";

function fragment() { return sql`WHERE active`; }
function indirect() { return fragment(); }
const arrow = () => sql`WHERE id = ${id}`;
const alias = arrow;
query(sql`SELECT id FROM accounts ${fragment()}`);
query(sql`SELECT id FROM accounts ${indirect()}`);
query(sql`SELECT id FROM accounts ${arrow()}`);
query(sql`SELECT id FROM accounts ${alias()}`);

function load(filter = sql`WHERE active`) {
  return query(sql`SELECT id FROM accounts ${filter}`);
}
function destructured({ clause = sql`WHERE active` } = {}) {
  const copy = clause;
  return query(sql`SELECT id FROM accounts ${copy}`);
}

query(sql`SELECT id FROM accounts ${sql["raw"]("WHERE active")}`);
query(sql`SELECT id FROM accounts ${fragment()["append"](" AND active")}`);

// Returning a plain string makes a parameter value, not a fragment.
function scalar() { return "active"; }
query(sql`SELECT id FROM accounts WHERE state = ${scalar()}`);
