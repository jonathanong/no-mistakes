import { query } from "@data-stores/psql";

const trap = function sql(strings: TemplateStringsArray, ...values: unknown[]) {
  return query(sql`SELECT * FROM topics WHERE id = ${values[0]}`);
};

export { trap };
