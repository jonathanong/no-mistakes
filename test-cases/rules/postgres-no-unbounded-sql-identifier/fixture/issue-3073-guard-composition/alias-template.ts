import { beginTransaction, type Query } from "db"
import { providers, type Provider, regions, type Region } from "@scope/pkg/config"

export async function load(provider: Provider, items: string[]) {
  if (items.length === 0) return []
  if (!providers.includes(provider)) throw new Error("unknown provider")
  const table = `${provider}_accounts`
  const captured = table
  const alias = captured
  const query = await beginTransaction()
  const first = await query(`/* first */ SELECT 1 FROM ${alias}`)
  const second = await query(`/* second */ SELECT 1 FROM ${alias}`)
  return [first, second]
}
