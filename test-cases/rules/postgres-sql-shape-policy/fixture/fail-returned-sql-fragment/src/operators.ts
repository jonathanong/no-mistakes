import { query } from '@example/db'
import sql from 'sql-template-strings'

export function extension(): string {
  return '.mjs'
}

function fhirOperatorToSqlOperator(): string {
  return '='
}

function wrapper(): string {
  return fhirOperatorToSqlOperator()
}

function lockMode(): string {
  return 'FOR UPDATE'
}

function isolation(): string {
  return 'REPEATABLE READ'
}

sql`SELECT 1 WHERE a ${wrapper()} b`

sql`SELECT 1`.append(lockMode())

query(`SET TRANSACTION ISOLATION LEVEL ${isolation()}`)
