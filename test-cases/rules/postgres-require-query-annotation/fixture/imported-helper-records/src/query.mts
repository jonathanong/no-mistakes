import { read } from '@bench/db'
import { named, missing, builder, mutable } from './helpers.mjs'
import type { named as typedNamed } from './helpers.mjs'
import { forwarded } from './reexport.mjs'
import { absent } from 'unavailable-package'

const plain = named(1)
read(plain.text, plain.values)
const absentName = missing()
read(absentName.text, absentName.values)
const tagged = builder(2)
read(tagged.text, tagged.values)
const changed = mutable()
read(changed.text, changed.values)
const forwardedResult = forwarded()
read(forwardedResult.text, forwardedResult.values)
const external = absent()
read(external.text, external.values)
const typeOnly = typedNamed(3)
read(typeOnly.text, typeOnly.values)
{
  const named = unknown
  const shadow = named(4)
  read(shadow.text, shadow.values)
}

import { pureRecord } from './plain-only.mjs'
const pure = pureRecord(5)
read(pure.text, pure.values)
