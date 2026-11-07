import { beforeAll, afterAll, it } from 'vitest'
import { readdirSync } from 'node:fs'
const previous = process.env.CASE_ROOT
beforeAll(() => { process.env.CASE_ROOT = process.argv[2] })
afterAll(() => { process.env.CASE_ROOT = previous })
it('reads data', () => { readdirSync(process.env.CASE_ROOT) })
