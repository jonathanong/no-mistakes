import { vi } from 'vitest'
import { forbidden } from './forbidden.mts'
vi.mock('./self.mts')
forbidden()
