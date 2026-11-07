import { readdirSync } from 'node:fs'
import { join } from 'node:path'
export function dataRoot() { return process.env.CASE_ROOT ?? './tmp/data' }
export function list() { return readdirSync(join(dataRoot(), 'table')) }
