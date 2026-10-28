import { read } from '@bench/db'
import sql from 'sql-template-strings'
import { continueHead, closeComment } from './other.mjs'

// These roots need suffix content before any head decision is sound.
const transaction = continueHead(sql`BEGIN`)
read(transaction.text, transaction.values) // low: false
const comment = closeComment(sql`/*`)
read(comment.text, comment.values) // low: false
const empty = continueHead(sql``).append(sql` trailing`)
read(empty.text, empty.values) // low: false

const longerWord = continueHead(sql`BEGINNING`)
read(longerWord.text, longerWord.values) // low: false
const named = continueHead(sql`/* stable */ SELECT `).append(sql` trailing`)
read(named.text, named.values) // low: false
const missing = continueHead(sql`SELECT `)
read(missing.text, missing.values) // low: false
