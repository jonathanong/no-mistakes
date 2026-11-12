import sql from 'sql-template-strings';
export function plain() { return {text: '/* record */ SELECT 1', values: []}; }
export function builder() { return sql`/* builder */ SELECT 2`; }
