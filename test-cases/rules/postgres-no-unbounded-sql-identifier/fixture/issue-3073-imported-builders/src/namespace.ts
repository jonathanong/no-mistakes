import { type QueryExecutor } from 'db'
import * as statements from './builders'

export const select = (execute: QueryExecutor) => execute(`/* namespace */ SELECT ${statements.statement.text}`, statements.statement.values)
