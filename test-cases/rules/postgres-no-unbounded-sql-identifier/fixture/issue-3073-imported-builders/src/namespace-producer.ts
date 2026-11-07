import sql from 'sql-template-strings'
export const namedWrite = sql`id FROM items`
export const namespaceWrite = sql`id FROM items`
export const untouched = sql`id FROM items`
