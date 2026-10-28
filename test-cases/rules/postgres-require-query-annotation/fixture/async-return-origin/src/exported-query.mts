import { read } from '@bench/db'
import { named } from './helpers.mjs'
const query = await named('user', {})
read(query)
export { query }
