import templateSql from "sql-template-strings";
import { query, sql } from "@example/db";

const value = "literal'quote";
const relation = "widgets";
const column = "name";
const sortColumn = "slug";
const condition = "name IS NOT NULL";
const subquery = "SELECT id FROM widgets LIMIT 1";

export function run() {
  query(sql`SELECT id FROM widgets WHERE id = (${subquery})`);
  query(sql`SELECT id FROM widgets WHERE slug LIKE ${value} AND name ILIKE ${value}`);
  query(sql`SELECT id FROM widgets WHERE slug BETWEEN ${value} AND ${value}`);
  query(sql`SELECT id FROM widgets WHERE slug BETWEEN 'a' AND 'z' AND ${condition}`);
  query(sql`SELECT name || ${value} FROM widgets`);
  query(sql`INSERT INTO widgets (${sortColumn}, ${column}) VALUES (${value}, ${value}), (${value}, ${value})`);
  query(sql`SELECT id FROM widgets WHERE (slug, name) > (${value}, ${value})`);
  query(sql`SELECT score + ${"1"} FROM widgets LIMIT ${"2"} OFFSET ${"3"}`);
  query(sql`SELECT ${column} FROM ${relation} AS w ORDER BY ${sortColumn}, w.${column}`);
  query(templateSql`SELECT ${column} FROM widgets`);
  query(sql`INSERT INTO widgets (slug, name) VALUES ((${value}), (${value}))`);
  query(sql`SELECT id FROM widgets WHERE (slug, name) > ((${value}), (${value}))`);
}
