import { vi } from 'vitest'
import { run } from './external-through.mts'
vi.mock('external-analyzer')
run()
