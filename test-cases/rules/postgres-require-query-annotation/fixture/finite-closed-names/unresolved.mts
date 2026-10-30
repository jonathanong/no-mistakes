import { read } from '@bench/db'
import { enabled } from './missing.mts'

const name = enabled ? 'accounts' : 'sessions'
read(`/* ${name} */ SELECT 1`)

read(`/* ${enabled ? 'posts' : 'comments'} */ SELECT 1`)

for (const table of ['accounts', 'sessions']) {
  read(`/* ${table} */ SELECT 1`)
}
