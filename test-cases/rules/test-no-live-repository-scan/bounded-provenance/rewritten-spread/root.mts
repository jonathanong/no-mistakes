import fs, { mkdtemp } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { vi } from 'vitest'
const root = await mkdtemp(join(tmpdir(), 'case-'))
const migrations = fileURLToPath(new URL('./migrations', import.meta.url))
const original = fs.readdir
vi.spyOn(fs, 'readdir').mockImplementation((...args) => {
 if (args[0] === migrations) args[0] = root
 return original(...args)
})
await fs.readdir(migrations)
