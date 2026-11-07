import { beginTransaction, type Query } from "db"
import { providers, type Provider, regions, type Region } from "@scope/pkg/config"

export async function unchecked(provider: Provider) {
  const table = `${provider}_accounts`
  const query = await beginTransaction()
  const first = await query(`/* first */ SELECT 1 FROM ${table}`)
  return query(`/* second */ SELECT 1 FROM ${table}`)
}
export async function asserted(provider: string) {
  if (!providers.includes(((provider as Provider)))) throw new Error("unknown provider")
  const table = `${provider}_accounts`
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${table}`)
  return query(`/* second */ SELECT 1 FROM ${table}`)
}
export async function openHole(provider: Provider, suffix: string) {
  if (!providers.includes(provider)) throw new Error("unknown provider")
  const table = `${provider}_${suffix}`
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${table}`)
  return query(`/* second */ SELECT 1 FROM ${table}`)
}
export async function capturedBeforeGuard(provider: Provider) {
  const table = `${provider}_accounts`
  if (!providers.includes(provider)) throw new Error("unknown provider")
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${table}`)
  return query(`/* second */ SELECT 1 FROM ${table}`)
}
export async function changedAfterGuard(provider: Provider) {
  if (!providers.includes(provider)) throw new Error("unknown provider")
  provider = process.env.PROVIDER as Provider
  const table = `${provider}_accounts`
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${table}`)
  return query(`/* second */ SELECT 1 FROM ${table}`)
}
export async function opaqueAddition(provider: Provider, suffix: string) {
  if (!providers.includes(provider)) throw new Error("unknown provider")
  const table = provider + (suffix as unknown as string).toString()
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${table}`)
  return query(`/* second */ SELECT 1 FROM ${table}`)
}
export async function mutatedCondition(provider: Provider, replacement: Provider) {
  if (!providers.includes(provider) || (provider = replacement, false)) throw new Error("unknown provider")
  const table = `${provider}_accounts`
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${table}`)
  return query(`/* second */ SELECT 1 FROM ${table}`)
}
export async function insufficientConjunction(provider: Provider, flag: boolean) {
  if (!providers.includes(provider) && flag) throw new Error("unknown provider")
  const table = `${provider}_accounts`
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${table}`)
  return query(`/* second */ SELECT 1 FROM ${table}`)
}
export async function callbackMutation(provider: Provider, replacement: string) {
  const reset = () => { provider = replacement as Provider; return false }
  if (!providers.includes(provider) || reset()) throw new Error("unknown provider")
  const table = `${provider}_accounts`
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${table}`)
  return query(`/* second */ SELECT 1 FROM ${table}`)
}
export async function compoundCallbackMutation(provider: Provider, replacement: string) {
  const reset = () => { provider += replacement; return false }
  if (!providers.includes(provider) || reset()) throw new Error("unknown provider")
  const table = `${provider}_accounts`
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${table}`)
  return query(`/* second */ SELECT 1 FROM ${table}`)
}
export async function hoistedCallbackMutation(provider: Provider) {
  if (!providers.includes(provider)) throw new Error("unknown provider")
  reset()
  const table = `${provider}_accounts`
  function reset() { provider = process.env.PROVIDER as Provider }
  const query = await beginTransaction()
  await query(`/* first */ SELECT 1 FROM ${table}`)
  return query(`/* second */ SELECT 1 FROM ${table}`)
}
