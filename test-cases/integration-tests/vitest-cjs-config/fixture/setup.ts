import { vi } from 'vitest'

vi.mock('external-service', () => ({ ok: true }))
