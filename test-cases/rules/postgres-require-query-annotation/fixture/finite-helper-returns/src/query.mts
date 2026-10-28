import { read } from '@bench/db'
import { named, missing, mixed, switched } from './helpers.mjs'

read(named(true))
read(missing(true))
read(mixed(true))
read(switched('a'))
read(named(true).append(tail))
const local = named(true)
read(local)
