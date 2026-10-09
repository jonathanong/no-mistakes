import sql from 'sql-template-strings';
export function diamondStatement() { return sql`/* diamond binding */ SELECT 1`; }
