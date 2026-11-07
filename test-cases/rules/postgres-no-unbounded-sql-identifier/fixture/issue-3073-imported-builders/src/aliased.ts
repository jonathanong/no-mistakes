import { type QueryExecutor } from 'db'
import { aliasedStatement } from './barrel'

export const select = (execute: QueryExecutor) => execute(`/* barrel */ SELECT ${aliasedStatement.text}`, aliasedStatement.values)
