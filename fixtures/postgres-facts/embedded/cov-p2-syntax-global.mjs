import { read } from '@example/db'
const replacement = { freeze(value) { return value } }
Object = replacement
Object.freeze({ query: read }) // UNKNOWN:read
read('SELECT 1') // SQL:read('SELECT 1')=>SELECT 1
