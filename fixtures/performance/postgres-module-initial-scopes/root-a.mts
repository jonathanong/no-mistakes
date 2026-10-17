import { read } from '@bench/db'
import { sql } from './shared.mts'
export function firstRoot() { return read(sql()) }
