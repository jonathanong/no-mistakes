import { beginTransaction, type Query } from "db"
import { providers, type Provider, regions, type Region } from "@scope/pkg/config"

export async function load(provider: Provider, region: Region) {
  if (!providers.includes(provider)) throw new Error("unknown provider")
  if (!regions.includes(region)) throw new Error("unknown region")
  const prefix = "tenant_"
  const base = `${prefix}${provider}_${region}`
  const table = base + `${"_accounts"}_${2}`
  const alias = table
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${alias}`)
  return query(`/* second */ SELECT 1 FROM ${alias}`)
}
