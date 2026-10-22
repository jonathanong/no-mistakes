import { vi } from 'vitest'

vi.mock('./static-target.mts', () => ({ value: 'static' }))
