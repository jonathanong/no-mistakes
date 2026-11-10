import { mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { list } from './list.mts'
const modulePath = process.argv[2]
await import(modulePath)
const root = mkdtempSync(join(tmpdir(), 'case-'))
list(root)
