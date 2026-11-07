import { beginTransaction, type Query } from "db"
import { providers, type Provider, regions, type Region } from "@scope/pkg/config"

export async function load(provider: Provider, blocked: boolean) {
  if (!providers.includes(provider) || blocked) throw new Error("unavailable")
  const table = `${provider}_accounts`
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${table}`)
  return query(`/* second */ SELECT 1 FROM ${table}`)
}
export async function loadReverse(provider: Provider, blocked: boolean) {
  if (blocked || !providers.includes(provider)) throw new Error("unavailable")
  const table = provider + "_accounts"
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${table}`)
  return query(`/* second */ SELECT 1 FROM ${table}`)
}
