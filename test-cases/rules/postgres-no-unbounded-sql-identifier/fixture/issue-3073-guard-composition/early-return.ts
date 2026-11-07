import { beginTransaction, type Query } from "db"
import { providers, type Provider, regions, type Region } from "@scope/pkg/config"

export async function load(provider: Provider) {
  if (!providers.includes(provider)) return []
  const suffix = "_accounts"
  const table = provider + suffix
  const alias = table
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${alias}`)
  return query(`/* second */ SELECT 1 FROM ${alias}`)
}
