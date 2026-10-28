import { read } from '@bench/db'
import sql from 'sql-template-strings'
import { selectWithOwner, wrongReturn, aliasEscape, unsafeDefault } from './helpers.mjs'
import { forwarded } from './reexport.mjs'

const accepted = selectWithOwner(sql`/* returnedParam */ SELECT `).append(sql` WHERE r.id = ${1}`)
read(accepted.text, accepted.values)

const wrong = wrongReturn(sql`/* wrongReturn */ SELECT `)
read(wrong.text, wrong.values) // unanalyzable:

const escaped = aliasEscape(sql`/* escaped */ SELECT `)
read(escaped.text, escaped.values) // unanalyzable:

const defaulted = unsafeDefault(sql`/* defaulted */ SELECT `)
read(defaulted.text, defaulted.values) // unanalyzable:

// An arbitrary extra object can have a getter that changes the first builder.
const getter = selectWithOwner(sql`/* getter */ SELECT `, { get columns() { return bad() }, join: sql`` })
read(getter.text, getter.values) // unanalyzable:

const redirected = forwarded(sql`/* redirected */ SELECT `)
read(redirected.text, redirected.values) // unanalyzable:

const unknown = selectWithOwner(sql`SELECT `)
read(unknown.text, unknown.values) // finding:
