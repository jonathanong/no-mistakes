import type { QueryExecutor, QueryOptions } from "@example/executor-types";
declare const query: QueryExecutor;
declare const options: QueryOptions;
declare const statement: string;
declare const flag: boolean;
declare function opaque(value: unknown): boolean;

// Tests inspect receiver metadata, while body invocations still execute SQL.
while (options.query) {
  break;
} // NON_SQL
do {
  break;
} while (options.query); // NON_SQL
for (; options.query;) {
  break;
} // NON_SQL
for (;;) {
  break;
} // NON_SQL
while (
  opaque(query) // ADVISE_OPAQUE_GUARD
) {
  break;
}
for (
  ;
  query(statement); // ADVISE_SQL_GUARD
) {
  break;
}
const absent = !options.query; // NON_SQL
const selected = flag ? options.query : query; // NON_SQL
selected(statement); // ADVISE_CONDITIONAL
const required = options.query!; // NON_SQL
required(statement); // ADVISE_NON_NULL
