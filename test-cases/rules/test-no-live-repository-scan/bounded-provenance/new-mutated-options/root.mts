import { mkdtempSync, readdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
function list(options: { root: string }) {
 new options.__defineGetter__('root', () => process.cwd())
 return readdirSync(options.root)
}
const root = mkdtempSync(join(tmpdir(), 'case-'))
list({ root })
