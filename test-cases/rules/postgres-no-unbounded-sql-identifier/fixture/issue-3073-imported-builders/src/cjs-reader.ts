import { type QueryExecutor } from 'db'
import { requiredStatement } from './cjs-producer'

export const readRequired = (execute: QueryExecutor) => execute(`/* CommonJS mutation */ SELECT ${requiredStatement.text}`, requiredStatement.values)
