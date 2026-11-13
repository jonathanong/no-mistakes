import sql from 'sql-template-strings';
export function plain() { return {text: '/* record */ SELECT 1', values: []}; }
export function builder() { return sql`/* builder */ SELECT 2`; }
export function named() { return '/* named */ SELECT 1'; }
export function returned(q) { const result = q.append(' LIMIT 1'); return result; }
