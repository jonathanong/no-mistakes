import { connection } from 'db'
import { createRequest } from './request.mjs'

export function run(sql: string) {
  createRequest().get('/users').query({ name: 'alice' })
  connection.query(sql)
  connection.query('SELECT id FROM users OFFSET 1')
  // no-mistakes-disable-next-line postgres-no-offset
  connection.query(sql)
}
