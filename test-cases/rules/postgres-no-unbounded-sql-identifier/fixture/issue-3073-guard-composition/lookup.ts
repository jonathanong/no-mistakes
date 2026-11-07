import { beginTransaction, type Query } from "db"
import { providers, type Provider, regions, type Region } from "@scope/pkg/config"

const tables = { facebook: "facebook_accounts", github: "github_accounts" } as const
export async function load(provider: Provider) {
  const table = tables[provider]
  if (!table) throw new Error("unknown provider")
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${table}`)
  return query(`/* second */ SELECT 1 FROM ${table}`)
}
