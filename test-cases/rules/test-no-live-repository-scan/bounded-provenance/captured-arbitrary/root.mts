import { readdirSync } from 'node:fs'
export function list(root: string) {
 function visit() { return readdirSync(root) }
 return visit()
}
