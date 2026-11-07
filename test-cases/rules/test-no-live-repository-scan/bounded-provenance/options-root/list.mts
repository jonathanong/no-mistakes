import { readdirSync } from 'node:fs'
export function list(options: { root: string }) { return readdirSync(options.root) }
