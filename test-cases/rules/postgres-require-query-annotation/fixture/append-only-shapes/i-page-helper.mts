import sql from 'sql-template-strings'
export async function pageHelper(options, message, name, kind, query, run) {
  query.append(sql` LIMIT ${options.limit}`)
  return run(query)
}
export async function opaquePage(options, message, name, kind, query, run) {
  options.prepare(query)
  return run(query)
}
export async function mutatingPage(options, message, name, kind, query, run) {
  query.strings[0] = 'SELECT 2'
  return run(query)
}
export async function replacingPage(options, message, name, kind, query, run) {
  return run(sql`SELECT 2`)
}
