import { read, type QueryExecutor } from '@example/db';

export const helpers = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
helpers.load(read);
const defaulted = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
export default defaulted;
defaulted.load(read);
const separate = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
export { separate };
separate.load(read);
