import { write } from "@app/db";

const other = { sql: (strings: TemplateStringsArray) => strings[0] };
const { sql } = other;

export function destructuredAliasShadowsDefaultTag() {
  return write(sql`/* destructured shadow */ SELECT 1`); // unanalyzable:destructured-shadow
}
