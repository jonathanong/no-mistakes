import sql from 'sql-template-strings'
export function appendDueSource(q, now) {
  q.append(sql` FROM due WHERE due.at <= ${now}`)
}
