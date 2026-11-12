import { read, write, type QueryExecutor, type OtherExecutor } from '@example/db';
function first(query: QueryExecutor) { query('SELECT 1'); }
function second(query: OtherExecutor) { query('SELECT 2'); }
first(read ?? write); // UNKNOWN_FIRST_MIXED_PROFILE
second(write ?? read); // UNKNOWN_SECOND_MIXED_PROFILE
const candidate = read || write;
first(candidate); // UNKNOWN_ALIASED_MIXED_PROFILE
