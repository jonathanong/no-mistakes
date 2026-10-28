import sql from 'sql-template-strings'
export function append(query, tail) { query.append(tail) }
export function extra(query) { query.append(sql` suffix`) }
export function unsafe(query) { query.text = 'SELECT unsafe' }
