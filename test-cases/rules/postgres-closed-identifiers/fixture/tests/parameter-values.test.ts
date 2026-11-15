import { query, sql } from "@example/db";
import { assertWhitelistedSqlIdentifier as allowed } from "@example/identifiers";

export function parameterizedValue(value: "pg_sleep(1)" | "DROP TABLE users") {
  return query(sql`/* parameterizedValue */ SELECT ${value}`);
}

export function parameterizedAssertion(value: string) {
  return query(
    sql`/* parameterizedAssertion */ SELECT ${allowed(value, ["pg_sleep(1)", "DROP TABLE users"])}`,
  );
}
