import { type QueryExecutor } from 'db'
import { untouched } from './namespace-producer'
import { grouped } from './namespace-barrel'
import * as all from './namespace-barrel'

export const named = () => grouped.untouched.text
export const nested = () => all.grouped.untouched.values
export const select = (execute: QueryExecutor) => execute(`/* untouched namespace */ SELECT ${untouched.text}`, untouched.values)
