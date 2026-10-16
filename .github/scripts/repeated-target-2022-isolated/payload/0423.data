import { read } from '@example/db'
import { buildSql } from './async-builder.ts'

export async function pendingRead() {
  return read(buildSql())
}

export async function awaitedRead() {
  return read(await buildSql())
}
