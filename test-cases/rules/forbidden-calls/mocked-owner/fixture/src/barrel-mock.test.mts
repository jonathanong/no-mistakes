import { vi } from 'vitest'
import { run } from './barrel-through.mts'
vi.mock('no-mistakes')
run()
