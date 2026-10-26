import { read } from "@example/db";
import type { QueryExecutor } from "@example/executor-types";
declare const query: QueryExecutor;
declare const dynamicSql: string;
declare function opaque(value: unknown): void;

// Holes and rest must preserve executor taint without blaming the declaration.
const [first, , ...rest] = [query, 0, query]; // NON_SQL
first(dynamicSql); // ADVISE_ARRAY_FIRST
rest[0](dynamicSql); // ADVISE_ARRAY_REST

// A default is another possible source, even when the destructured source is empty.
const [arrayDefault = query] = []; // NON_SQL
arrayDefault(dynamicSql); // ADVISE_ARRAY_DEFAULT
const { query: objectDefault = query } = {}; // NON_SQL
objectDefault(dynamicSql); // ADVISE_OBJECT_DEFAULT
const {
  nested: { query: nestedQuery },
} = { nested: { query } }; // NON_SQL
nestedQuery(dynamicSql); // ADVISE_NESTED_PATTERN
const { plain, ...remaining } = { plain: 0, query }; // NON_SQL
remaining.query(dynamicSql); // ADVISE_OBJECT_REST

// A neighboring scalar element/property must not become an executor.
const [, scalar] = [query, 1]; // NON_SQL
opaque(scalar); // NON_SQL
opaque(plain); // NON_SQL
read("SELECT 1"); // CLEAR_LITERAL
