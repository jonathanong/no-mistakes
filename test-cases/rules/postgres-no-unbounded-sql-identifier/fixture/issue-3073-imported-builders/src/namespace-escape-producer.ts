import sql from 'sql-template-strings'
export const bareStatement = sql`id FROM items`
export const aliasStatement = sql`id FROM items`
export const quotedStatement = sql`id FROM items`
export const opaqueStatement = sql`id FROM items`
export const quotedNamedStatement = sql`id FROM items`
export const relatedSafeStatement = sql`id FROM items`
