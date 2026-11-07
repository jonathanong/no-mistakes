import fs from 'node:fs/promises'
import { mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { vi } from 'vitest'
const root = mkdtempSync(join(tmpdir(), 'case-'))
const original = fs.readdir
vi.spyOn(fs, 'readdir').mockImplementation((...args) => {
 if (args[0] === 'migrations') args[0] = root
 return original(...args)
})
await fs.readdir(process.argv[2])
