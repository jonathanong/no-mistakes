import { read } from '@bench/db'
import { named, interpolated, missing, partial, mutable, aliased } from './values.mjs'
import type { named as typedNamed } from './values.mjs'
import { forwarded } from './reexport.mjs'
import { duplicate } from './ambiguous.mjs'
import { absent } from 'unavailable-package'

read(named)
read(interpolated)
read(aliased)
read(missing)
read(partial)
read(mutable)
read(forwarded)
read(duplicate)
read(absent)
read(typedNamed)
{
  const named = dynamic
  read(named)
}
read(named?.text)
