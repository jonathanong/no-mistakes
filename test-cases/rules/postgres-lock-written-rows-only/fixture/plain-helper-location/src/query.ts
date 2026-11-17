import { PoolClient } from '@example/db'

export async function main(client: PoolClient) {
  await lockDdl(client)
  await lockDdl(client)
  await lockDdl(client)
}

async function lockDdl(client: PoolClient) {
  await client.query(`SELECT pg_advisory_lock(hashtextextended('k',0))`)
}

async function suppressedLockDdl(client: PoolClient) {
  // no-mistakes-disable-next-line postgres-lock-written-rows-only
  await client.query(`SELECT pg_advisory_lock(hashtextextended('k',0))`)
}

export async function suppressed(client: PoolClient) {
  await suppressedLockDdl(client)
  await suppressedLockDdl(client)
  await suppressedLockDdl(client)
}
