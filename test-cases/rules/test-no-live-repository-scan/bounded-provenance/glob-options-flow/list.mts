import { globSync } from 'node:fs'
import { resolve } from 'node:path'
export function list(root: string, patterns: string[]) {
 const directory = resolve(root, 'specs')
 return patterns.flatMap(pattern => globSync(pattern, { cwd: directory }))
}
