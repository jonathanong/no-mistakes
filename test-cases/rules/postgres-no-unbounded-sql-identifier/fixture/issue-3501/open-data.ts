export const OPEN_TABLES = loadTables();
export const MUTABLE_TABLES = ['orders', 'refunds'];
MUTABLE_TABLES.push(loadTable());
export function assertIdent(value: string) { return value; }
