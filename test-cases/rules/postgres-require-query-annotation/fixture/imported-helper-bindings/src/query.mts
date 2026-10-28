import { read } from '@bench/db'
import sql from 'sql-template-strings'
import { named, missing, plain, returnedLocal, mutable } from './helpers.mjs'
import { forwarded } from './reexport.mjs'
import { absent } from 'unavailable-package'

const a = named(1)
read(a)
const b = missing()
read(b)
const c = returnedLocal(2)
c.append(sql` WHERE true`)
read(c)
read(named(3).append(sql` WHERE true`))
read(missing().append(sql` WHERE true`))
read(plain().append(sql` WHERE true`))
const d = plain()
read(d)
const e = forwarded()
read(e)
const f = absent()
read(f)
const g = mutable()
read(g)
