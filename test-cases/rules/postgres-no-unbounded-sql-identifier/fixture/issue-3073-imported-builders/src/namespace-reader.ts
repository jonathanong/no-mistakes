import { type QueryExecutor } from 'db'
import { namedWrite, namespaceWrite, untouched } from './namespace-producer'
import './namespace-writes'

export const named = (execute: QueryExecutor) => execute(`/* namespace export write */ SELECT ${namedWrite.text}`, namedWrite.values)
export const nested = (execute: QueryExecutor) => execute(`/* nested namespace write */ SELECT ${namespaceWrite.text}`, namespaceWrite.values)
export const safe = (execute: QueryExecutor) => execute(`/* other namespace field */ SELECT ${untouched.text}`, untouched.values)
