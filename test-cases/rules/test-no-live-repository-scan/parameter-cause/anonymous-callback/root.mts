import { readdirSync } from 'node:fs'
['a'].forEach((root: string) => readdirSync(root))
