import { connection, openTransaction, type TxExecutor } from '@example/db'
import { createRequest } from './request.mjs'

export function direct(sql: string) {
  return connection.query(sql)
}

export function alias(sql: string) {
  const db = connection
  return db['query'](sql)
}

export function typed(db: TxExecutor, sql: string) {
  return db.query(sql)
}

export function factory(sql: string) {
  const db = openTransaction()
  db.query(sql)
  {
    // The nested local shadows the configured factory result.
    const db = { query: (_value: object) => undefined }
    db.query({ sql })
  }
}

export function unrelated(sql: string) {
  const request = createRequest().get('/users')
  request.query({ sql })
  createRequest().get('/users').query({ sql })
}

export function shadow(connection: { query: (value: object) => unknown }) {
  return connection.query({ value: 1 })
}
