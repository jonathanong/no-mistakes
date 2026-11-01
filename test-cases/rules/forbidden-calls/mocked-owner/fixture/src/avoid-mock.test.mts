import { vi } from 'vitest'
import { run } from './avoid-through.mts'
vi.mock('no-mistakes')
run()
