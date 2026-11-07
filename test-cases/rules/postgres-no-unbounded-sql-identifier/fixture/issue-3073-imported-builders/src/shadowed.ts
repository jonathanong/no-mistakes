import { type QueryExecutor } from 'db'
import * as statements from './builders'

export function shadowed(statements: any, execute: QueryExecutor) {
  return execute(`/* shadowed */ SELECT ${statements.statement.text}`, statements.statement.values)
}
