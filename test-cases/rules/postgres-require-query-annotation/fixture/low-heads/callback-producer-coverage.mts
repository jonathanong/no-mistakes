import sql from 'sql-template-strings'
export function unnamed(base, execute) { execute(sql`SELECT 1`) }
