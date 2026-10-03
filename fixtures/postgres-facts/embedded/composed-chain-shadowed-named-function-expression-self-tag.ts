import { query } from "@example/db";

const trap = function sql(strings: TemplateStringsArray, ...values: unknown[]) {
  return query(sql`SELECT * FROM topics WHERE id = ${values[0]}`);
};

export { trap };
