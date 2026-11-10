import { readdirSync } from 'node:fs'

export default (root: string) => readdirSync(root)
