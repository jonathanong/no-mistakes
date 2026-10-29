import sql from 'sql-template-strings'
import other from 'other-sql'

// Repeated export names must share both successful and failed source censuses.
function factory(select, extra, from = sql``) {
  const statement = select.append(sql` suffix`)
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}
export { factory as factory0 }
export { factory as factory1 }
export { factory as factory2 }
export { factory as factory3 }
export { factory as factory4 }
export { factory as factory5 }
export { factory as factory6 }
export { factory as factory7 }
export { factory as factory8 }
export { factory as factory9 }
export { factory as factory10 }
export { factory as factory11 }
export { factory as factory12 }
export { factory as factory13 }
export { factory as factory14 }
export { factory as factory15 }
factory(sql`/* local0 */ SELECT `)
factory(sql`/* local1 */ SELECT `)
factory(sql`/* local2 */ SELECT `)
factory(sql`/* local3 */ SELECT `)
factory(sql`/* local4 */ SELECT `)
factory(sql`/* local5 */ SELECT `)
factory(sql`/* local6 */ SELECT `)
factory(sql`/* local7 */ SELECT `)
factory(sql`/* local8 */ SELECT `)
factory(sql`/* local9 */ SELECT `)
factory(sql`/* local10 */ SELECT `)
factory(sql`/* local11 */ SELECT `)
factory(sql`/* local12 */ SELECT `)
factory(sql`/* local13 */ SELECT `)
factory(sql`/* local14 */ SELECT `)
factory(sql`/* local15 */ SELECT `)
factory(sql`/* local16 */ SELECT `)
factory(sql`/* local17 */ SELECT `)
factory(sql`/* local18 */ SELECT `)
factory(sql`/* local19 */ SELECT `)
factory(sql`/* local20 */ SELECT `)
factory(sql`/* local21 */ SELECT `)
factory(sql`/* local22 */ SELECT `)
factory(sql`/* local23 */ SELECT `)
// Repeated export names must share both successful and failed source censuses.
function untrusted(select, extra, from = other``) {
  const statement = select.append(sql` suffix`)
  if (extra) statement.append(extra.columns)
  statement.append(from)
  if (extra) statement.append(extra.join)
  return statement
}
export { untrusted as untrusted0 }
export { untrusted as untrusted1 }
export { untrusted as untrusted2 }
export { untrusted as untrusted3 }
export { untrusted as untrusted4 }
export { untrusted as untrusted5 }
export { untrusted as untrusted6 }
export { untrusted as untrusted7 }
export { untrusted as untrusted8 }
export { untrusted as untrusted9 }
export { untrusted as untrusted10 }
export { untrusted as untrusted11 }
export { untrusted as untrusted12 }
export { untrusted as untrusted13 }
export { untrusted as untrusted14 }
export { untrusted as untrusted15 }
untrusted(sql`/* local0 */ SELECT `)
untrusted(sql`/* local1 */ SELECT `)
untrusted(sql`/* local2 */ SELECT `)
untrusted(sql`/* local3 */ SELECT `)
untrusted(sql`/* local4 */ SELECT `)
untrusted(sql`/* local5 */ SELECT `)
untrusted(sql`/* local6 */ SELECT `)
untrusted(sql`/* local7 */ SELECT `)
untrusted(sql`/* local8 */ SELECT `)
untrusted(sql`/* local9 */ SELECT `)
untrusted(sql`/* local10 */ SELECT `)
untrusted(sql`/* local11 */ SELECT `)
untrusted(sql`/* local12 */ SELECT `)
untrusted(sql`/* local13 */ SELECT `)
untrusted(sql`/* local14 */ SELECT `)
untrusted(sql`/* local15 */ SELECT `)
untrusted(sql`/* local16 */ SELECT `)
untrusted(sql`/* local17 */ SELECT `)
untrusted(sql`/* local18 */ SELECT `)
untrusted(sql`/* local19 */ SELECT `)
untrusted(sql`/* local20 */ SELECT `)
untrusted(sql`/* local21 */ SELECT `)
untrusted(sql`/* local22 */ SELECT `)
untrusted(sql`/* local23 */ SELECT `)
