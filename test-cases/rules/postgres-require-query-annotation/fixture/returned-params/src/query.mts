import { read } from '@bench/db'
import sql from 'sql-template-strings'
import { selectWithOwner, wrongReturn, aliasEscape, unsafeDefault, continueHead, closeComment } from './helpers.mjs'
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

// EOF transaction heads and incomplete comments cannot survive arbitrary suffixes.
const transaction = continueHead(sql`BEGIN`)
read(transaction.text, transaction.values) // unanalyzable:
const partialComment = closeComment(sql`/*`)
read(partialComment.text, partialComment.values) // unanalyzable:
const emptyHead = continueHead(sql``).append(sql` trailing`)
read(emptyHead.text, emptyHead.values) // unanalyzable:

// Stable heads remain provable with the inert empty native default.
const longerWord = continueHead(sql`BEGINNING`)
read(longerWord.text, longerWord.values) // finding:
const namedEmptyDefault = continueHead(sql`/* emptyDefault */ SELECT `).append(sql` trailing`)
read(namedEmptyDefault.text, namedEmptyDefault.values)
const missingEmptyDefault = continueHead(sql`SELECT `)
read(missingEmptyDefault.text, missingEmptyDefault.values) // finding:
