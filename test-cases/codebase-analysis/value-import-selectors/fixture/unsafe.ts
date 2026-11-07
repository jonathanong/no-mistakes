import { statement } from './source.ts'
import * as namespace from './source.ts'
declare function consume(value: unknown): void
statement.text = process.env.SQL
consume(statement)
const alias = statement
namespace.statement.text = process.env.SQL
consume(namespace.statement)
