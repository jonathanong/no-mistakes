import { vi } from 'vitest'

vi.mock('../module')
const server = await import('msw/node')
