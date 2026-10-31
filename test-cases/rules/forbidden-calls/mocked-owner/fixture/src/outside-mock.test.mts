import { vi } from 'vitest'
vi.mock('../hidden/outside.mts', () => ({}))
vi.mock('external-analyzer-factory', () => ({}))
