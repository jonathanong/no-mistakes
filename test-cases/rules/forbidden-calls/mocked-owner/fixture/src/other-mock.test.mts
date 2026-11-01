import { vi } from 'vitest'
import { run } from './other-through.mts'
vi.mock('./unrelated.mts')
run()
