import { read } from "@example/db";
import type { QueryExecutor } from "@example/executor-types";
declare const query: QueryExecutor;
declare const dynamicSql: string;
declare function opaque(value: unknown): void;
declare function unconfigured(): unknown;

// Safe numeric and null keys must preserve the same executable property value.
const keyed = { 0: query, null: query, hostname: "local" }; // NON_SQL
const numericQuery = keyed[0]; // NON_SQL
numericQuery(dynamicSql); // ADVISE_NUMERIC_KEY
const nullQuery = keyed["null"]; // NON_SQL
nullQuery(dynamicSql); // ADVISE_NULL_KEY
opaque(keyed.hostname); // NON_SQL

// Reading a nested object must retain its origin and contained executor.
const nested = { value: { query }, hostname: "local" }; // NON_SQL
opaque(nested.value); // ADVISE_NESTED_ORIGIN
opaque(nested.hostname); // NON_SQL

// A producer is a callable escape, while an unrelated property remains data.
function producer(): QueryExecutor {
  return unconfigured() as QueryExecutor; // NON_SQL
}
const producers = { make: producer, hostname: "local" }; // NON_SQL
opaque(producers.make); // ADVISE_PRODUCER_PROPERTY
opaque(producers.hostname); // NON_SQL

// An outer unrelated assertion cannot erase the inner executor provenance.
const assertedFields = {
  asField: unconfigured() as QueryExecutor as unknown, // NON_SQL
  satisfiesField: unconfigured() as QueryExecutor satisfies unknown, // NON_SQL
  angleField: <unknown>(<QueryExecutor>unconfigured()), // NON_SQL
  run() {
    this.asField(dynamicSql); // ADVISE_AS_FIELD
    this.satisfiesField(dynamicSql); // ADVISE_SATISFIES_FIELD
    this.angleField(dynamicSql); // ADVISE_ANGLE_FIELD
  },
}; // NON_SQL
const assertedAlias = unconfigured() as QueryExecutor satisfies unknown; // NON_SQL
assertedAlias(dynamicSql); // ADVISE_ASSERTED_ALIAS
read("SELECT 1"); // CLEAR_LITERAL
