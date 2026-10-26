import { beginTransaction, read } from '@example/db';
import type { QueryOptions } from '@example/executor-types';
declare const dynamicSql: string;
declare function opaque(value: unknown): void;

const options = { query: read } satisfies QueryOptions;
// A configured factory receives metadata without executing that executor.
const query = beginTransaction(options); // NON_SQL
query('SELECT 1'); // CLEAR_FACTORY_LITERAL
query(dynamicSql); // ADVISE_FACTORY_DYNAMIC

// Unsupported destructuring does not gain literal proof from factory identity.
const { query: extracted } = beginTransaction(options); // NON_SQL
extracted('SELECT 1'); // ADVISE_FACTORY_DESTRUCTURED
extracted(dynamicSql); // ADVISE_FACTORY_DESTRUCTURED_DYNAMIC
opaque(beginTransaction(options)); // ADVISE_FACTORY_ESCAPE
