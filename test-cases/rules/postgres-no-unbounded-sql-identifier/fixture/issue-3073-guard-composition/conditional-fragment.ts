import { beginTransaction } from "db"

const providers = ["facebook", "github"] as const
type Provider = (typeof providers)[number]
export async function load(provider: Provider, accountId: string | undefined) {
  if (!providers.includes(provider)) throw new Error("unknown provider")
  const accountTable = `${provider}_accounts`
  const accountFilter = accountId ? "AND id = $2::uuid" : ""
  const params = [0, accountId]
  const query = await beginTransaction()
  await query(`SELECT id FROM ${accountTable} WHERE expires_at < $1 ${accountFilter} FOR UPDATE SKIP LOCKED`, params)
  await query(`SELECT id FROM ${accountTable} WHERE expires_at < $1 FOR UPDATE SKIP LOCKED`, [0])
  const filter = accountFilter
  const order = accountId ? "ORDER BY id" : ""
  await query(`SELECT id FROM ${accountTable} WHERE expires_at < $1 ${filter} ${order} FOR UPDATE SKIP LOCKED`, params)
  const statement = `SELECT id FROM ${accountTable} WHERE expires_at < $1 ${accountId ? "AND id = $2::uuid" : ""} FOR UPDATE SKIP LOCKED`
  await query(statement, params)
  await query(`SELECT id FROM ${accountTable} WHERE expires_at < $1 ${accountId ? "AND id = $2::uuid" : ""} FOR UPDATE SKIP LOCKED`, params)
  const present = "AND id = $2::uuid"
  const absent = ""
  const unionFilter = accountId ? present : absent
  await query(`SELECT id FROM ${accountTable} WHERE expires_at < $1 ${unionFilter} FOR UPDATE SKIP LOCKED`, params)
  const firstColumn = accountId ? "id" : "url"
  const secondColumn = accountId ? "state" : "status"
  await query(`SELECT ${firstColumn}, ${secondColumn} FROM ${accountTable} WHERE expires_at < $1 ${accountFilter} ${order} FOR UPDATE SKIP LOCKED`, params)
  await query(`SELECT id FROM ${accountTable}2 WHERE expires_at < $1 ${accountFilter} FOR UPDATE SKIP LOCKED`, params)
  await query(`SELECT id FROM ${accountTable} WHERE (expires_at < $1 ${accountFilter}) FOR UPDATE SKIP LOCKED`, params)
  await query(`WITH selected AS (SELECT id FROM ${accountTable} WHERE expires_at < $1 ${accountFilter}) SELECT id FROM selected FOR UPDATE SKIP LOCKED`, params)
}
