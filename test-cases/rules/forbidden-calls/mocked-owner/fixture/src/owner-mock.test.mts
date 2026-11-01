import { vi } from 'vitest'
import { run } from './through-owner.mts'
vi.mock('no-mistakes')
run()
