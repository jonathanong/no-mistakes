import { mkdtemp, readdir } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { describe, it } from 'vitest'
describe('owned', () => {
 const roots: string[] = []
 async function tempRoot() {
  const root = await mkdtemp(join(tmpdir(), 'case-'))
  roots.push(root)
  return root
 }
 it('lists owned output', async () => {
  const root = await tempRoot()
  await readdir(root)
 })
})
