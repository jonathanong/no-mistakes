import { mkdtempSync, readdirSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
function workspace() {
 const root = mkdtempSync(join(tmpdir(), 'case-'))
 return { root, output: join(root, 'output') }
}
const { root, output } = workspace()
readdirSync(output)
