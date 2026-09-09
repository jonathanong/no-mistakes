import tag from "sql-template-strings";
import { query } from "@data-stores/psql";

// Nested destructuring must shadow the imported tag alias.
export function load(
  provider: { tag: (strings: TemplateStringsArray, ...values: unknown[]) => string },
  id: number,
) {
  const { tag } = provider;
  return query(tag`SELECT * FROM topics WHERE id = ${id}`);
}
