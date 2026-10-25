import { read } from '@example/db';
import type { QueryOptions } from '@example/executor-types';
declare const dynamicSql: string;
declare function opaque(value: unknown): void;
declare const options: QueryOptions;

// A proven global freeze constructs data and does not execute the contained executor.
const frozenOptions = Object.freeze({ ...options }); // NON_SQL
const hostname = frozenOptions.hostnameId; // NON_SQL
hostname.toString(); // NON_SQL

// Executor identity and taint remain available at actual calls after projection.
frozenOptions.query(dynamicSql); // ADVISE_FROZEN_QUERY
const detachedQuery = frozenOptions.query; // NON_SQL
detachedQuery(dynamicSql); // ADVISE_DETACHED_QUERY
const scientificKey = Object.freeze({ '1e+21': options.query }); // NON_SQL
const detachedScientificQuery = scientificKey[1e21]; // NON_SQL
detachedScientificQuery(dynamicSql); // ADVISE_SCIENTIFIC_KEY
const frozenFunction = Object.freeze(read); // NON_SQL
frozenFunction(dynamicSql); // ADVISE_FROZEN_FUNCTION
opaque(Object.freeze(read)); // ADVISE_FROZEN_FUNCTION_ESCAPE
opaque(() => frozenOptions.query(dynamicSql)); // ADVISE_FROZEN_CALLBACK

// Frozen containers still escape when an opaque caller can execute their values.
opaque(Object.freeze({ query: options.query })); // ADVISE_FROZEN_CONTAINER
const nested = Object.freeze({ nested: { query: options.query } }); // NON_SQL
opaque(nested); // ADVISE_NESTED_CONTAINER
const frozenArray = Object.freeze([options.query]); // NON_SQL
opaque(frozenArray); // ADVISE_FROZEN_ARRAY
frozenArray.forEach(query => query(dynamicSql)); // ADVISE_ARRAY_CALLBACK
const frozenMethod = Object.freeze({
  query: options.query,
  run() { this.query(dynamicSql); },
}); // NON_SQL
frozenMethod.run(); // ADVISE_FROZEN_METHOD
