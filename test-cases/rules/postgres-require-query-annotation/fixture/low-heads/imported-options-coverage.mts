import { read } from '@bench/db'
import { factory } from './unknown.mjs'
const query = factory()
read(query.text, query.values, { limited: true }) // low: false
