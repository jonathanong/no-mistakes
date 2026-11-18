import { read } from '@example/db'
import { PROVIDERS } from './providers.mts'

type Provider = (typeof PROVIDERS)[number]

export function opaqueAddition(provider: Provider, suffix: string) {
  if (!PROVIDERS.includes(provider)) throw new Error('unknown provider')
  const table = provider + (suffix as unknown as string).toString()
  read(`/* first */ SELECT 1 FROM ${table}`)
  return read(`/* second */ SELECT 1 FROM ${table}`)
}

export function aliasedReceiver(provider: Provider, suffix: string) {
  if (!PROVIDERS.includes(provider)) throw new Error('unknown provider')
  const receiver = suffix
  const table = provider + receiver.toString()
  read(`/* first */ SELECT 1 FROM ${table}`)
  return read(`/* second */ SELECT 1 FROM ${table}`)
}

export function unwrappedSuffix(provider: Provider, suffix: string) {
  if (!PROVIDERS.includes(provider)) throw new Error('unknown provider')
  const table = provider + suffix
  read(`/* first */ SELECT 1 FROM ${table}`)
  return read(`/* second */ SELECT 1 FROM ${table}`)
}

export function finiteReceiver(provider: Provider, suffix: Provider) {
  if (!PROVIDERS.includes(provider)) throw new Error('unknown provider')
  const table = provider + suffix.toString()
  read(`/* first */ SELECT 1 FROM ${table}`)
  return read(`/* second */ SELECT 1 FROM ${table}`)
}

export function literalReceiver(provider: Provider) {
  if (!PROVIDERS.includes(provider)) throw new Error('unknown provider')
  const table = provider + '_events'.toString()
  read(`/* first */ SELECT 1 FROM ${table}`)
  return read(`/* second */ SELECT 1 FROM ${table}`)
}
