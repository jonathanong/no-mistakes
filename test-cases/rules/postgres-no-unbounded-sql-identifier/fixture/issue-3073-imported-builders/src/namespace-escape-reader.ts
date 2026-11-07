import { type QueryExecutor } from 'db'
import { bareStatement, aliasStatement, quotedStatement, opaqueStatement, quotedNamedStatement, relatedSafeStatement } from './namespace-escape-producer'
import './namespace-escapes'

export const bare = (execute: QueryExecutor) => execute(`/* bare escape */ SELECT ${bareStatement.text}`, bareStatement.values)
export const alias = (execute: QueryExecutor) => execute(`/* namespace alias */ SELECT ${aliasStatement.text}`, aliasStatement.values)
export const quoted = (execute: QueryExecutor) => execute(`/* quoted member */ SELECT ${quotedStatement.text}`, quotedStatement.values)
export const opaque = (execute: QueryExecutor) => execute(`/* opaque member */ SELECT ${opaqueStatement.text}`, opaqueStatement.values)
export const named = (execute: QueryExecutor) => execute(`/* quoted named export */ SELECT ${quotedNamedStatement.text}`, quotedNamedStatement.values)
export const safe = (execute: QueryExecutor) => execute(`/* unrelated namespace */ SELECT ${relatedSafeStatement.text}`, relatedSafeStatement.values)
