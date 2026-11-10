import { mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { vi } from 'vitest'
import { list } from './list.mts'
vi.mock('./list.mts')
const root = mkdtempSync(join(tmpdir(), 'case-'))
list(root)
