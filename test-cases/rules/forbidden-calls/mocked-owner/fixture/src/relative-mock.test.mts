import { vi } from 'vitest'
import { run } from './nested/through.mts'
vi.mock('./local-owner.mts')
run()
