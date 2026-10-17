import { read } from '@bench/db'
import { sql } from './shared.mts'
export function secondRoot() { return read(sql()) }
