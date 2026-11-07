import { mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { beforeAll, afterAll, it } from 'vitest'
import { list } from './list.mts'
let previous: string | undefined
beforeAll(() => {
 previous = process.env.CASE_ROOT
 process.env.CASE_ROOT = mkdtempSync(join(tmpdir(), 'case-'))
})
afterAll(() => {
 if (previous === undefined) delete process.env.CASE_ROOT
 else process.env.CASE_ROOT = previous
})
it('reads owned data', () => { list() })
