import { read } from '@bench/db'
import { remote } from './remote.mjs'
const result = await remote()
read(result)
eval('opaque')
